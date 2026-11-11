use super::{audio::media_player::MediaPlayer, source};
use crate::logic::data_types::artist::Artist;
use crate::logic::data_types::event_context::EventContext;
use crate::logic::data_types::playlist::Playlist;
use crate::logic::data_types::queue_item::QueueItem;
use crate::logic::data_types::setting::Setting;
use crate::logic::data_types::source::Source;
use crate::logic::data_types::state::SearchResults;
use crate::logic::data_types::track::Track;
use crate::logic::database;
use crate::logic::queue::Queue;
use crate::logic::validate_sources;
use crate::{AppWindow, ContextMenuActions, MainView, MediaActions, SearchActions, SettingActions, SlintState, State};
use slint::{ComponentHandle};
use std::{cell::RefCell, rc::Rc};

// Todo: restructure the whole file and stream line it. Too messy to work with after a while.

pub fn setup(app: &AppWindow, state: &Rc<RefCell<State>>, rt: &tokio::runtime::Runtime) {
    handle_initialization(&mut state.borrow_mut());
    println!("Playlists: {:?}", &state.borrow().playlists);
    handle_passing_values(app, &mut state.borrow_mut());
    handle_events(app, state, rt);
}

fn handle_initialization(state: &mut State) {
    // Initialize database or restore previous session
    if database::initialize_tables().is_ok() {
        println!("Database initialized");
        if let Ok(list) = database::get_table::<Track>() {
            println!("Fetched most recent tracks: {:?}", list);
            state.index = list.into_iter().map(|t| Rc::new(RefCell::new(t))).collect();
        }

        if let Ok(list) = database::get_table::<QueueItem>() {
            println!("Fetched most recent queue: {:?}", list);
            state.queue = list;
        }

        if let Ok(list) = database::get_table::<Playlist>() {
            println!("Fetched most recent playlists: {:?}", list);
            state.playlists = list;
        }

        let settings = Vec::from([
       		Setting {
        		name: "volume".to_string(),
          		value: "-1".to_string(),
            	default_value: "20".to_string(),
            	updated_at: "".to_string(),
             	created_at: "".to_string(),
        	}
        ]);
        if database::add_records(settings).is_ok() {
      		println!("Settings set");
       	}
    } else {
        println!("Couldn't create db connection for initialization");
    }

    // Update, incase something changed during initialization
    if let Ok(read_sources) = validate_sources() {
        println!("Checked files {:?}", &read_sources);
        let _ = database::add_records(read_sources);
        println!("Updated file sources");
        if let Ok(list) = database::get_table::<Track>() {
            println!("Files: {:?}", list);
            state.index = list.into_iter().map(|t| Rc::new(RefCell::new(t))).collect();
        }
        if let Ok(lists) = database::get_table::<Playlist>() {
        	state.playlists = lists;
        }
        if let Ok(settings) = database::get_table::<Setting>() {
        	if let Some(volume) = settings.iter().find(|s| s.name == "volume") {
         		if let Some(numeric) = Setting::is_numeric(&volume.value) {
	         		if numeric != -1 {
	        			state.volume = numeric as f32;
	           		} else {
						if let Some(numeric) = Setting::is_numeric(&volume.default_value) {
	             			state.volume = numeric as f32;
						}
	             	}
	         	}
           	}
        }
    }
}

fn handle_passing_values(app: &AppWindow, state: &mut State) {
    let global_state = app.global::<SlintState>();
    state.set_index(Some(state.index.clone()), &global_state);
    state.set_queue(Some(state.queue.clone()), &global_state);
    state.set_playlist(&global_state);
    state.set_volume(&global_state);
}

fn handle_events(app: &AppWindow, state: &Rc<RefCell<State>>, _tokio_runtime: &tokio::runtime::Runtime) {
    let global_media_actions = app.global::<MediaActions>();
    let global_setting_actions = app.global::<SettingActions>();
    let global_search_actions = app.global::<SearchActions>();
    let global_context_menu_actions = app.global::<ContextMenuActions>();
    let context = EventContext {
    	app: app.as_weak(),
     	state: state.clone(),
      	player: Rc::new(RefCell::new(MediaPlayer::new(state.clone())))
    };

    // Media elements bottom panel.
    global_media_actions.on_media_start({
       	let context = context.clone();

        move |id: i32| {
            println!("Media start triggered! On track id : {}", &id);
            context.with_state(|state, _, player| {
           		state.index_playing_reset();
            	if let Some((index, track)) = state.index
	                .iter()
	                .enumerate()
	                .find(|(_, item)| item.borrow().id == id) {
		             	track.borrow_mut().playing = true;

						let c = context.clone();
						let p = player.clone();
						let t = track.clone();

			            slint::spawn_local(async move {
			                audio_control_events::handle_media_start(p, t.clone())
			                    .await;
							c.with_state(|s, a, _| {
								s.timeline.set_timeline(&t, a);
							});
			            })
						.ok();

		                state.timeline.media_index = Some(index);
		                let temp_queue = state.queue.clone();
		                if let Some((queue_index, _)) = temp_queue
		                    .iter()
		                    .enumerate()
		                    .find(|(_, queue_item)| queue_item.track_id == track.borrow().id)
		                {
		                    state.timeline.queue_index = Some(queue_index);
		                }
			        }
            });
        }
    });
    // TODO: Rewrite
    // global_media_actions.on_media_change({
    //     let app_clone = weak_app.clone();
    //     let state_clone = Rc::clone(state);
    //     let mut player_clone = Rc::clone(&player);

    //     move |index: i32| {
    //         println!("(UI Events) Media changed");
    //         let queue_result = state_clone
    //             .borrow_mut()
    //             .update_playing_audio_in_queue(index);

    //         if let Some(app) = app_clone.upgrade() {
    //             state_clone
    //                 .borrow_mut()
    //                 .set_queue(None, &app.global::<SlintState>());
    //         }

    //         if let Some((previous_index, target_index)) = queue_result {
    //             let is_empty = state_clone.borrow().queue.is_empty();
    //             if !is_empty {
    //                 let id = state_clone.borrow().queue[target_index].track_id;
    //                 if let Some((_, media)) = state_clone.borrow().find_source_by_id(id) {
    //                     audio_control_events::handle_media_change(
    //                         &mut player_clone,
    //                         media,
    //                         (previous_index, target_index),
    //                     );
    //                 }
    //             }
    //         }
    //     }
    // });
    global_media_actions.on_media_pause({
        let context = context.clone();

        move || {
        	context.with_state(|_, _, player| {
		       	if !player.borrow().is_paused() {
	        		player.borrow_mut().pause();
	        	}
         	});
        }
    });
    global_media_actions.on_media_toggle({
        let context = context.clone();

        move || {
        	context.with_state(|_, _, player| {
		       	if !player.borrow().is_paused() {
	        		player.borrow_mut().unpause();
	        	} else {
					player.borrow_mut().pause();
				}
        	});
        }
    });
    global_media_actions.on_media_change_volume({
    	let context = context.clone();

        move |volume: f32| {
        	context.with_state(|state, _, player| {
	        	state.volume = volume.clone();
		        player.borrow_mut().set_volume(volume);
         	});
        }
    });
    global_media_actions.on_media_change_track_position({
        let context = context.clone();

        move |position: i32| {
        	context.with_state(|_, _, player| {
            	let duration_position = position as f64;
            	player.borrow_mut().try_seek(duration_position);
         	});
        }
    });
    global_media_actions.on_media_get_track_position({
   		let context = context.clone();

        move || {
        	context.with_state(|state, app, player| {
	            println!("(Event) Get track triggered");
	            let value = player.borrow_mut().get_position();
	            state.timeline.update_timeline(value as i32, app);
         	});
        }
    });
    global_media_actions.on_media_mix({
    	let context = context.clone();

        move || {
        	context.with_state(|state, _, player| {
	            state.shuffle();
	            if player.borrow().is_rnged() {
	                player.borrow_mut().set_queue_rng(false);
	            } else {
	                player.borrow_mut().set_queue_rng(true);
	            }
         	});
        }
    });
    global_media_actions.on_add_to_queue({
    	let context = context.clone();

        move |id: i32| {
       		context.with_state(|state, global, _| {
         		state.add_to_queue(id, true, global);
         	});
        }
    });
    global_media_actions.on_add_to_playlist({
    	let context = context.clone();

        move |playlist_id: i32, track_id: i32| {
            context.with_state(|state, app, _| {
                state.add_to_playlist(
                    playlist_id,
                    track_id,
                    app,
                );
            });
        }
    });

    global_setting_actions.on_new_local_source({
    	let context = context.clone();

        move || {
            let source: Option<std::path::PathBuf> = source::new_local_source();
            match source {
                Some(source) => {
                    {
                        let path_string = source.clone().to_str().unwrap().to_string();
                        let _ = database::add_record(Source {
                            origin: String::from("local"),
                            path: path_string,
                        });

                        // TODO: Add logic that goes through the errors and checks if the track has been replaced.
                        // Example 1: Artist 1 track 1 is replaced with Artist 2 track 1, with the same name.
                        // If we can't find Artist 1's track 1, we can remove it from the db and add Artist 2's track 1 without a problem.
                        // Example 2: If the program scans and check the db doesn't include old tracks before adding new tracks, we can prevent the unique path error.
                    }

                    println!("Directory fetched correctly {:?}", source);
                    // if database::add_records(records.clone()).is_ok() {
                    // 	state_clone.borrow_mut().merge_to_index(records);
                    // }

                    let records = source::read_source(source).expect("Couldn't fetch all files");
                    if database::add_records(records.clone()).is_ok() {
                    	context.with_state(|state, app, _| {
                        	state.merge_to_index(records);
                            state.set_index(None, app);
                     	});
                    };
                }
                None => println!("Didn't receive a path. Result should be None: {:?}", source),
            }
        }
    });

    global_search_actions.on_view_change({
    	let context = context.clone();

		move |main_view: MainView, unique_key: i32| {
			context.with_state(|state, app, _| {
				let value = crate::SlintSelected {
		     		main_view: main_view,
		       		selected_id: unique_key,
		     	};

				state.set_main_view(value, app);
			});
		}
    });

    global_search_actions.on_view_change_generic({
  		let context = context.clone();

		move |main_view: MainView| {
			context.with_state(|state, app, _| {
				let value = crate::SlintSelected {
					main_view: main_view,
					// To indicate that the view was changed with intention
			  		selected_id: -2,
				};

				state.set_main_view(value, app);
			});
		}
    });

	// TODO: make sure the search_term is valid (shouldn't be trusted)
    global_search_actions.on_search_local({
    	let context = context.clone();

		// TODO: Extemely simple search result that probably is too slow for large indexes.
		// TODO: Tags should a hash map.
	    move |search_term: slint::SharedString, _tags: slint::ModelRc<slint::SharedString>| {
			context.with_state(|state, app, _| {
				let s_term = search_term.to_ascii_lowercase();
				let mut found_artists: Vec<String> = Vec::new();
				let tracks: Vec<Track> = state.index
					.iter()
					.filter(|track| {
						let t = track.borrow();
						let found = t.artist.to_ascii_lowercase().contains(&s_term);
						if found && !found_artists.contains(&s_term) {
							found_artists.push(track.borrow().artist.clone())
						}
						found && !found_artists.contains(&s_term)
					})
					.map(|track| track.borrow().clone())
					.collect();

				let mut artists: Vec<Artist> = Vec::new();
				if let Ok(list) = database::get_table::<Artist>() {
					artists = list
						.iter()
						.cloned()
						.filter(|artist| found_artists.contains(&artist.name))
						.collect();
				};

				let playlists: Vec<Playlist> = state.playlists
					.iter()
					.filter(|p| p.name.to_ascii_lowercase().contains(&s_term))
					.map(|p| p.clone())
					.collect();

				let results = SearchResults {
					tracks,
					artists,
					playlists: playlists,
				};

	        	state.set_search_results(results, app);
				println!("State search results changed: {:?}", state.search_result);
			});
	    }
    });

//     global_search_actions.on_search_online({
//     	use requests::Fetch;
// 	    let state_clone = Rc::clone(state);
// 		let tokio_handle = tokio_runtime.handle().clone();
// 		let app_clone = weak_app.clone();
//
// 	    move |search_term: slint::SharedString, tags: slint::ModelRc<slint::SharedString>| {
// 			let tokio_handle = tokio_handle.clone();
// 			let mut tracks: Vec<Track> = Vec::new();
//
// 			slint::spawn_local(async move {
// 				let result = tokio_handle.spawn(async move {
// 					requests::search(
// 						search_term.to_string().to_ascii_lowercase(),
// 						tags.iter().map(|s| s.to_string()).collect(),
// 						Fetch::Search
// 					).await
// 				})
// 				.await
// 				.unwrap();
//
// 				match result {
// 					Ok(SearchResults::Videos(videos)) =>
// 						tracks = videos.into_iter().map(Into::into).collect(),
// 					Ok(_) =>
// 						eprintln!("UI events :: enum logic not implemented to on_search_local()"),
// 					Err(e) =>
// 						eprintln!("Search failed: {e}")
// 				};
// 			});
//
//         	state_clone.borrow_mut().search_result = tracks;
// 	    }
//     });

    global_context_menu_actions.on_create_new_playlist({
        use crate::Playlist;
        let context = context.clone();

        move || {
            let playlists = database::get_table::<Playlist>();
            let playlist_count = if playlists.is_ok() {
                playlists.unwrap().len() + 1
            } else {
                0
            };
            let name = format!("Playlist {playlist_count}");

            let new_entry = Playlist {
                id: 0,
                name,
                artist: None,
                list_type: "playlist".to_string(),
                thumbnail: "".to_string(),
                created_at: "".to_string(),
                listened_at: "".to_string(),
                sources: None,
                tracks: None,
            };

            if database::add_record(new_entry.clone()).is_ok() {
            	context.with_state(|state, app, _| {
	                state.playlists.push(new_entry);
                    state.set_playlist(app);
             	});
            };
        }
    });
}

pub mod audio_control_events {
	use crate::{logic::{audio::media_player::MediaPlayer, data_types::track::Track}};
    use std::{cell::RefCell, rc::Rc};

    pub async fn handle_media_start(player: Rc<RefCell<MediaPlayer>>, track: Rc<RefCell<Track>>) {
        let _ = player.borrow_mut().play(&track).await;
        let timer = watch_track_ending(player.clone());
       	*player.borrow_mut().end_timer.lock().unwrap() = Some(timer);
    }

    fn watch_track_ending(player: Rc<RefCell<MediaPlayer>>) -> slint::Timer {
        let timer = slint::Timer::default();

        timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(250), move || {
            let finished = match player.borrow().handle.lock().unwrap().as_ref() {
                Some(handle) => matches!(handle.state(), kira::sound::PlaybackState::Stopped),
                None => true,
            };

            if finished {
                println!("Track finished");
                handle_media_change(player.clone(), (0, 0));
            }
        });

        timer
    }

    pub fn handle_media_change(player: Rc<RefCell<MediaPlayer>>, (previous_index, current_index): (usize, usize)) {
        // If the queue moved only by one, skip to next track
       	slint::spawn_local(async move {
	        if current_index == previous_index + 1 {
	            if let Ok(Some(track)) = player.borrow_mut().next() {
            		handle_media_start(player.clone(), track).await
	            }
	        }
       	}).unwrap();
        // Else the queue needs to be remade within the media player, if the queue is loopable.
    }

    pub fn handle_media_loop(player: Rc<RefCell<MediaPlayer>>) {
        println!("create_loop action triggered");
        if player.borrow().is_looped() {
            player.borrow_mut().set_queue_loop(false);
        } else {
            player.borrow_mut().set_queue_loop(true);
        }
    }
}
