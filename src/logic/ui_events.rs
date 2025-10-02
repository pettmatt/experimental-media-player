use crate::logic::{database::{self, MediaFile, Source}, validate_sources};
use crate::{AppWindow, SlintState, SettingActions, MediaActions, State};
use crate::logic::queue::Queue;
use super::{audio::media_player::MediaPlayer, source};
use std::{cell::RefCell, rc::Rc};
use slint::ComponentHandle;

pub fn handle_initialization(state: &mut State) {
	// Initialize database or restore previous session
	if database::initialize_tables().is_ok() {
		println!("Database initialized");
		if let Ok(list) = database::get_table::<MediaFile>() {
			println!("Fetched most recent details: {:?}", list);
			state.index = list;
		}
	} else {
		println!("Couldn't create db connection for initialization")
	}

	// Update, incase something changed during initialization
	if let Ok(read_sources) = validate_sources() {
		println!("Checked files {:?}", &read_sources);
		database::add_records(read_sources);
		println!("Updated file sources");
		if let Ok(media_list) = database::get_table::<MediaFile>() {
			println!("Files: {:?}", media_list);
			state.index = media_list;
		}
	}
}

pub fn handle_passing_values(app: &AppWindow, state: &mut State) {
	let global_state = app.global::<SlintState>();
	state.set_index(Some(state.index.clone()), &global_state);
	state.set_queue(Some(state.queue.clone()), &global_state);
}

pub fn handle_events(app: &AppWindow, state: &mut Rc<RefCell<State>>) {
	let player = Rc::new(RefCell::new(MediaPlayer::new()));
	let global_setting_actions = app.global::<SettingActions>();
	let global_media_actions = app.global::<MediaActions>();

	let mut player_clone_1 = Rc::clone(&player);
	let mut player_clone_2 = Rc::clone(&player);
	let mut player_clone_3 = Rc::clone(&player);
	let mut player_clone_4 = Rc::clone(&player);
	let mut player_clone_5 = Rc::clone(&player);
	let mut player_clone_6 = Rc::clone(&player);
	let player_clone_7 = Rc::clone(&player);
	
	let mut state_clone_1 = Rc::clone(state);
	let mut state_clone_2 = Rc::clone(state);
	let state_clone_3 = Rc::clone(state);
	let state_clone_4 = Rc::clone(state);
	let state_clone_5 = Rc::clone(state);
	let state_clone_6 = Rc::clone(state);
	
	let weak_app = app.as_weak();
	let app_clone = weak_app.clone();

	// Media elements bottom panel.
	global_media_actions.on_media_start(move |id: i32| {
		let temp_state = state_clone_1.borrow().index.clone();
		if let Some((index, media)) = temp_state.iter().enumerate()
			.find(|(_, item)| item.id == id) {
				audio_control_events::handle_media_start(&mut player_clone_1, media);
				state_clone_1.borrow_mut().add_to_queue(media);

				if let Some(app) = app_clone.upgrade() {
					state_clone_1.borrow_mut().set_queue(None, &app.global::<SlintState>());
					state_clone_1.borrow_mut().playing.media_index = Some(index);

					let temp_queue = state_clone_1.borrow().queue.clone();
					if let Some((queue_index, _)) = temp_queue.iter().enumerate()
						.find(|(_, queue_item)| queue_item.media_id == media.id) {
							state_clone_1.borrow_mut().playing.queue_index = Some(queue_index);
						}
				}
		}
	});
	global_media_actions.on_media_change(move |index: i32| {
		println!("(UI Events) Media changed");
		let queue_result = state_clone_2
			.borrow_mut().update_playing_audio_in_queue(index);

		if let Some(app) = app_clone.upgrade() {
			state_clone_2.borrow_mut().set_queue(None, &app.global::<SlintState>());
		}

		if let Some((previous_index, target_index)) = queue_result {
			let is_empty = state_clone_2.borrow().queue.is_empty();
			if !is_empty {
				let id = state_clone_2.borrow().queue[target_index].media_id;
				if let Some((_, media)) = state_clone_2.borrow().find_source_by_id(id) {
					audio_control_events::handle_media_change(
						&mut player_clone_2,
						media,
						(previous_index, target_index)
					);
				}
			}
		}
	});
	global_media_actions.on_media_toggle(move ||
		audio_control_events::handle_media_toggle(&mut player_clone_3));
	global_media_actions.on_media_change_volume(move |volume: i32|
		audio_control_events::handle_media_volume(&mut player_clone_4, volume));
	global_media_actions.on_media_change_track_position(move |position| {
		let duration_position = std::time::Duration::from_secs_f32(position as f32);
		audio_control_events::change_current_track_position(
			&mut player_clone_5,
			duration_position
		);
	});
	global_media_actions.on_media_get_track_position(move || {
		println!("(Event) Get track triggered");
		let value = audio_control_events::get_current_track_position(& player_clone_6);
		state_clone_3.borrow_mut().playing.update_position(value as i32);
	});
	global_media_actions.on_media_mix(move || {
		state_clone_4.borrow_mut().shuffle();
		audio_control_events::handle_media_mix(&player_clone_7);
	});

	// Settings
	global_setting_actions.on_new_local_source(move || {
		let source: Option<std::path::PathBuf> = source::new_local_source();

		match source {
			Some(source) => {
				{
					let path_string = source.clone().to_str().unwrap().to_string();
					database::add_record(Source {
						origin: String::from("local"),
						path: path_string
					});
				}

				println!("Directory fetched correctly {:?}", source);
				let records = source::read_source(source).expect("Couldn't fetch all files");
				database::add_records(records.clone());
				state_clone_5.borrow_mut().merge_to_index(records);
			},
			None => println!("Didn't receive a path. Result should be None: {:?}", source)
		}
	});

	global_media_actions.on_add_to_playlist(move |playlist_id: i32, media_id: i32| {
		if let Some(app) = app_clone.upgrade() {
			state_clone_6.borrow_mut().add_playlist(playlist_id, media_id, &app.global::<SlintState>());
		}
	});
}

mod audio_control_events {
	use crate::{logic::{audio::media_player::MediaPlayer, database::{self, MediaFile, QueueItem}}, State};
	use std::{cell::RefCell, rc::Rc, time::Duration};

	pub fn handle_media_toggle(media_player: &mut Rc<RefCell<MediaPlayer>>) {
		media_player.borrow_mut().source_toggle();
	}

	pub fn handle_media_start(media_player: &mut Rc<RefCell<MediaPlayer>>, media: &MediaFile) {
		media_player.borrow_mut().start(media);
	}

	pub fn handle_media_change(
		media_player: &mut Rc<RefCell<MediaPlayer>>, media: &MediaFile,
		(previous_index, current_index): (usize, usize)
	) {
		// If the queue moved only by one, skip to next track
		let difference = previous_index.saturating_sub(current_index);
		if difference == 1 || difference == usize::MIN {
			media_player.borrow_mut().next();
		}
		// Else the queue needs to be remade within the media player
	}

	pub fn handle_media_loop(media_player: &mut Rc<RefCell<MediaPlayer>>) {
		println!("create_loop action triggered");
	}

	pub fn handle_media_mix(media_player: &Rc<RefCell<MediaPlayer>>) {}

	pub fn handle_media_volume(media_player: &mut Rc<RefCell<MediaPlayer>>, volume: i32) {
		println!("(event) Volume change action triggered");
		media_player.borrow().set_volume(volume as f32);
	}

	pub fn handle_add_media_queue(media_player: Rc<RefCell<MediaPlayer>>, record: &MediaFile, state: &mut State) {
		if let Ok(()) = media_player.borrow_mut().add_to_queue(state, record) {
			if let Some(media) = state.index
				.iter().find(|list_item| list_item.path == record.path) {
					database::add_record::<QueueItem>(QueueItem {
						media_id: media.id,
					});
				}
		} else {
			println!("Couldn't add to media queue");
		}
	}

	pub fn change_current_track_position(media_player: &mut Rc<RefCell<MediaPlayer>>, position: Duration) {
		media_player.borrow_mut().change_current_track_position(position);
	}

	pub fn get_current_track_position(media_player: &Rc<RefCell<MediaPlayer>>) -> u32 {
		media_player.borrow().get_current_track_position()
	}
}
