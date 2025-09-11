use custom::ErrorHandler;

mod audio;
mod source;
pub mod database;
mod custom {
	use thiserror::Error;

	#[derive(Error, Debug)]
	pub enum ErrorHandler {
		#[error("SQLite error: {0}")]
		Sqlite(#[from] rusqlite::Error),
		#[error("IO error: {0}")]
		Io(#[from] std::io::Error),
	}
}

pub mod ui {
	use crate::{logic::{database::{self, MediaFile, Source}, validate_sources}, slint_generatedAppWindow};
	use crate::{AppWindow, SlintState, SettingActions, MediaActions, State};
	use super::{audio::MediaPlayer, source};
	use std::{rc::Rc, cell::RefCell};
	use slint::{ComponentHandle, ModelRc};

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

	pub fn handle_passing_values(app: &AppWindow, state: &State) {
		let global_state = app.global::<SlintState>();
		let queue_items: Vec<slint_generatedAppWindow::SlintQueueItem> = state.queue.clone()
			.into_iter()
			.map(|q| slint_generatedAppWindow::SlintQueueItem {
				currently_playing: q.currently_playing,
				media_id: q.media_id,
			})
			.collect();

		let media_items: Vec<slint_generatedAppWindow::SlintMediaFile> = state.index.clone()
			.into_iter()
			.map(|m| slint_generatedAppWindow::SlintMediaFile {
				id: m.id,
				artist: m.artist.into(),
				extension: m.extension.into(),
				file_size: m.file_size,
				name: m.name.into(),
				path: m.path.into(),
			})
			.collect();

		let queue_model = ModelRc::from(&queue_items[..]);
		let media_model = ModelRc::from(&media_items[..]);

		global_state.set_queue(queue_model);
		global_state.set_index(media_model);
	}

	pub fn handle_events(app: &AppWindow, state: &State) {
		let mut player = Rc::new(RefCell::new(MediaPlayer::new()));
		let global_setting_actions = app.global::<SettingActions>();
		let global_media_actions = app.global::<MediaActions>();

		// Media elements bottom panel.
		let mut player_clone_1 = Rc::clone(&player);
		let mut player_clone_2 = Rc::clone(&player);
		let mut player_clone_3 = Rc::clone(&player);
		let mut player_clone_4 = Rc::clone(&player);
		// let player_clone_5 = Rc::clone(&player);

		let state_clone_1 = state.clone();

		global_media_actions.on_media_change(
			move |index: i32| audio_control_events::handle_media_change(&mut player_clone_1, index));
		global_media_actions.on_media_start(move |id: i32| {
			let audio: &Option<&MediaFile> = &state_clone_1.index.iter().find(|item| item.id == id);

			if let Some(media) = audio {
				audio_control_events::handle_media_start(&mut player_clone_2, media.clone());
			}
		});
		global_media_actions.on_media_toggle(move ||
			audio_control_events::handle_media_toggle(&mut player_clone_3));
		global_media_actions.on_media_loop(move ||
			audio_control_events::handle_media_loop(&mut player_clone_4));
		// global_media_actions.on_media_mix(move ||
		// 	audio_control_events::handle_media_mix(player_clone_5));

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
					// println!("Files read correctly {:?}", records);
					database::add_records(records);
				},
				None => println!("Didn't receive a path. Result should be None: {:?}", source)
			}
		});
	}

	mod audio_control_events {
		use crate::{logic::{audio::MediaPlayer, database::{self, MediaFile, QueueItem}}, State};
		use std::{rc::Rc, cell::RefCell};

		pub fn handle_media_toggle(media_player: &mut Rc<RefCell<MediaPlayer>>) {
			media_player.borrow_mut().media_toggle();
		}

		pub fn handle_media_start(media_player: &mut Rc<RefCell<MediaPlayer>>, media: &MediaFile) {
			media_player.borrow_mut().media_start(media);
		}

		pub fn handle_media_change(media_player: &mut Rc<RefCell<MediaPlayer>>, index: i32) {
			let move_to_next_audio = index > 0;
		}

		pub fn handle_media_loop(media_player: &mut Rc<RefCell<MediaPlayer>>) {
			println!("create_loop action triggered");
		}

		pub fn handle_media_mix(media_player: Rc<RefCell<MediaPlayer>>) {}

		pub fn handle_add_media_queue(media_player: Rc<RefCell<MediaPlayer>>, record: &MediaFile, state: &mut State) {
			if let Ok(()) = media_player.borrow_mut().add_to_queue(state, record) {
				if let Some(media) = state.index
					.iter().find(|list_item| list_item.path == record.path) {
						database::add_record::<QueueItem>(QueueItem {
							media_id: media.id,
							currently_playing: false,
						});
					}
			} else {
				println!("Couldn't add to media queue");
			}
		}
	}
}

// Note to self: Source and Database shouldn't interact with each other. Instead create functions
// that utilize some functionality of source to call a function in database or vice versa.

pub fn validate_sources() -> Result<Vec<database::MediaFile>, ErrorHandler> {
	let source_list = database::get_table::<database::Source>()?;
	source::validate_sources(source_list)
}
