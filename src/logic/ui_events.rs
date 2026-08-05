use super::{audio::media_player::MediaPlayer, source};
use crate::logic::data_types::source::Source;
use crate::logic::data_types::track::Track;
use crate::logic::database;
use crate::logic::queue::Queue;
use crate::logic::validate_sources;
use crate::{AppWindow, ContextMenuActions, MediaActions, SettingActions, SlintState, State};
use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};

// Todo: restructure the whole file and stream line it. Too messy to work with after a while.

pub fn handle_initialization(state: &mut State) {
    // Initialize database or restore previous session
    if database::initialize_tables().is_ok() {
        println!("Database initialized");
        if let Ok(list) = database::get_table::<Track>() {
            println!("Fetched most recent details: {:?}", list);
            state.index = list.into_iter().map(|t| Rc::new(RefCell::new(t))).collect();
        }
    } else {
        println!("Couldn't create db connection for initialization")
    }

    // Update, incase something changed during initialization
    if let Ok(read_sources) = validate_sources() {
        println!("Checked files {:?}", &read_sources);
        database::add_records(read_sources);
        println!("Updated file sources");
        if let Ok(list) = database::get_table::<Track>() {
            println!("Files: {:?}", list);
            state.index = list.into_iter().map(|t| Rc::new(RefCell::new(t))).collect();
        }
    }
}

pub fn handle_passing_values(app: &AppWindow, state: &mut State) {
    let global_state = app.global::<SlintState>();
    state.set_index(Some(state.index.clone()), &global_state);
    state.set_queue(Some(state.queue.clone()), &global_state);
    state.set_volume(&global_state);
}

pub fn handle_events(app: &AppWindow, state: &mut Rc<RefCell<State>>) {
    let player = Rc::new(RefCell::new(MediaPlayer::new(state.clone())));
    let global_media_actions = app.global::<MediaActions>();
    let global_setting_actions = app.global::<SettingActions>();
    let global_context_menu_actions = app.global::<ContextMenuActions>();
    let weak_app = app.as_weak();

    // Media elements bottom panel.
    global_media_actions.on_media_start({
        let app_clone = weak_app.clone();
        let state_clone = Rc::clone(state);
        let player_clone = Rc::clone(&player);
        let index_clone = state_clone.borrow().index.clone();

        move |id: i32| {
            println!("Media start triggered! On track id : {}", &id);
        	let timer = slint::Timer::default;
         	state_clone.borrow_mut().index_playing_reset();

            if let Some((index, track)) = index_clone
                .iter()
                .enumerate()
                .find(|(_, item)| item.borrow().id == id)
            {
            	track.borrow_mut().playing = true;
                let player_temp_clone = player_clone.clone();
                let track_temp_clone = track.clone();
                let track_for_timeline = track.clone();
                let app_timeline_clone = app_clone.clone();
                let state_timeline_clone = state_clone.clone();

                slint::spawn_local(async move {
                    audio_control_events::handle_media_start(player_temp_clone, track_temp_clone)
                        .await;
      		        if let Some(app) = app_timeline_clone.upgrade() {
             			let global_state = app.global::<SlintState>();
						state_timeline_clone.borrow_mut().timeline.set_timeline(track_for_timeline.clone(), global_state);
			        }
                })
                .unwrap();

                if let Some(app) = app_clone.upgrade() {
                	state_clone.borrow_mut().set_current_track(&app.global::<SlintState>());
                    state_clone.borrow_mut().timeline.media_index = Some(index);
                    let temp_queue = state_clone.borrow().queue.clone();
                    if let Some((queue_index, _)) = temp_queue
                        .iter()
                        .enumerate()
                        .find(|(_, queue_item)| queue_item.track_id == track.borrow().id)
                    {
                        state_clone.borrow_mut().timeline.queue_index = Some(queue_index);
                    }
                }
            }
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
        let mut player_clone = Rc::clone(&player);
        move || audio_control_events::handle_media_pause(&mut player_clone)
    });
    global_media_actions.on_media_toggle({
        let mut player_clone = Rc::clone(&player);
        move || audio_control_events::handle_media_toggle(&mut player_clone)
    });
    global_media_actions.on_media_change_volume({
        let mut player_clone = Rc::clone(&player);
        let mut state_clone = Rc::clone(state);
        move |volume: f32| audio_control_events::handle_media_volume(&mut player_clone, &mut state_clone, volume)
    });
    global_media_actions.on_media_change_track_position({
        let mut player_clone = Rc::clone(&player);
        move |position| {
            let duration_position = position as f64;
            audio_control_events::change_current_track_position(&mut player_clone, duration_position);
        }
    });
    global_media_actions.on_media_get_track_position({
    	let app_clone = weak_app.clone();
        let state_clone = Rc::clone(state);
        let player_clone = Rc::clone(&player);

        move || {
            println!("(Event) Get track triggered");
        	if let Some(app) = app_clone.upgrade() {
	            let value = audio_control_events::get_current_track_position(&player_clone);
	            let global_state = app.global::<SlintState>();
	            state_clone
	                .borrow_mut()
	                .timeline
	                .update_timeline(value as i32, global_state);
         	}
        }
    });
    global_media_actions.on_media_mix({
        let state_clone = Rc::clone(state);
        let player_clone = Rc::clone(&player);

        move || {
            state_clone.borrow_mut().shuffle();
            audio_control_events::handle_media_mix(&player_clone);
        }
    });
    global_media_actions.on_add_to_queue({
        let state_clone = Rc::clone(&state);
        move |id: i32| audio_control_events::handle_add_media_queue(&state_clone, id)
    });
    global_media_actions.on_add_to_playlist({
        let app_clone = weak_app.clone();
        let state_clone = Rc::clone(state);

        move |playlist_id: i32, media_id: i32| {
            if let Some(app) = app_clone.upgrade() {
                state_clone.borrow_mut().add_to_playlist(
                    playlist_id,
                    media_id,
                    &app.global::<SlintState>(),
                );
            }
        }
    });

    global_setting_actions.on_new_local_source({
        let state_clone = Rc::clone(state);
        let app_clone = weak_app.clone();

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
                        state_clone.borrow_mut().merge_to_index(records);
                        if let Some(app) = app_clone.upgrade() {
                            let global_state = app.global::<SlintState>();
                            state_clone.borrow_mut().set_index(None, &global_state);
                        }
                    };
                }
                None => println!("Didn't receive a path. Result should be None: {:?}", source),
            }
        }
    });

    global_context_menu_actions.on_create_new_playlist({
        use crate::Playlist;
        let app_clone = weak_app.clone();
        let state_clone = Rc::clone(state);

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
                image_url: "".to_string(),
                created_at: "".to_string(),
                listened_at: "".to_string(),
                sources: None,
                tracks: None,
            };

            if database::add_record(new_entry.clone()).is_ok() {
                state_clone.borrow_mut().playlists.push(new_entry);
                if let Some(app) = app_clone.upgrade() {
                    let global_state = app.global::<SlintState>();
                    state_clone.borrow_mut().set_new_playlist(&global_state);
                }
            };
        }
    });
}

pub mod audio_control_events {
    use crate::{
        logic::{audio::media_player::MediaPlayer, data_types::track::Track},
        State,
    };
    use std::{cell::RefCell, rc::Rc};

    pub fn handle_media_toggle(media_player: &mut Rc<RefCell<MediaPlayer>>) {
        if media_player.borrow().is_paused() {
            media_player.borrow_mut().unpause();
        } else {
            media_player.borrow_mut().pause();
        }
    }

    pub fn handle_media_pause(media_player: &mut Rc<RefCell<MediaPlayer>>) {
    	if !media_player.borrow().is_paused() {
     		media_player.borrow_mut().pause();
     	}
    }

    pub async fn handle_media_start(media_player: Rc<RefCell<MediaPlayer>>, track: Rc<RefCell<Track>>) {
        let _ = media_player.borrow_mut().play(track).await;
        let watch_p = media_player.clone();
        let timer = watch_track_ending(watch_p, media_player.borrow().generation,
       	);
        media_player.clone().borrow_mut().end_timer = Some(timer);

    }

    fn watch_track_ending(media_player: Rc<RefCell<MediaPlayer>>, generation: u64) -> slint::Timer {
        let timer = slint::Timer::default();
        let timer_p = media_player.clone();

        timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(250), move || {
            let p = timer_p.borrow();
            if p.generation != generation {
                return;
            }

            let finished = match p.handle.as_ref() {
                Some(handle) => matches!(handle.state(), kira::sound::PlaybackState::Stopped),
                None => true,
            };

            if finished {
                println!("Track finished");
                drop(p);
                handle_media_change(media_player.clone(), (0, 0));
            }
        });

        timer
    }

    pub fn handle_media_change(media_player: Rc<RefCell<MediaPlayer>>, (previous_index, current_index): (usize, usize)) {
        // If the queue moved only by one, skip to next track
        let difference = previous_index.saturating_sub(current_index);
        if difference == 1 || difference == usize::MIN {
            if let Ok(Some(track)) = media_player.borrow_mut().next() {
            	handle_media_start(media_player.clone(), track);
            }
        }
        // Else the queue needs to be remade within the media player, if the queue is loopable.
    }

    pub fn handle_media_loop(media_player: &mut Rc<RefCell<MediaPlayer>>) {
        println!("create_loop action triggered");
        if media_player.borrow().is_looped() {
            media_player.borrow_mut().set_queue_loop(false);
        } else {
            media_player.borrow_mut().set_queue_loop(true);
        }
    }

    pub fn handle_media_mix(media_player: &Rc<RefCell<MediaPlayer>>) {
        if media_player.borrow().is_rnged() {
            media_player.borrow_mut().set_queue_rng(false);
        } else {
            media_player.borrow_mut().set_queue_rng(true);
        }
    }

    pub fn handle_media_volume(media_player: &Rc<RefCell<MediaPlayer>>, state: &Rc<RefCell<State>>, volume: f32) {
        println!("(event) Volume change action triggered");
        state.borrow_mut().volume = volume.clone();
        media_player.borrow_mut().set_volume(volume);
    }

    pub fn handle_add_media_queue(state: &Rc<RefCell<State>>, id: i32) {
        // Add logic that uses different append logic if index is included.
        let was_added = state.borrow_mut().add_to_queue(id, true);
        if !was_added {
            // Prompt user with a box if they want to add the track again.
            // state.add_to_queue(id, true);
        } else {
            // Do something if necessary to reflect changes on the UI.
        }
    }

    pub fn change_current_track_position(
        media_player: &mut Rc<RefCell<MediaPlayer>>,
        position: f64,
    ) {
        media_player.borrow_mut().try_seek(position);
    }

    pub fn get_current_track_position(media_player: &Rc<RefCell<MediaPlayer>>) -> f64 {
        media_player.borrow_mut().get_position()
    }
}
