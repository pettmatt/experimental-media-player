use std::collections::HashMap;
use custom::ErrorHandler;
use database::{MediaFile, Source};

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
	use crate::{logic::database::{self, Source}, AppWindow};
	use super::source;

	pub fn handle_events(app: &AppWindow) {
		// Media elements bottom panel.
		app.on_media_change(move |index: i32| audio_control_events::handle_media_change(index));
		app.on_media_start(move |start: bool| audio_control_events::handle_media_start(start));
		app.on_media_loop(move |create_loop: bool| audio_control_events::handle_media_loop(create_loop));
		app.on_media_mix(audio_control_events::handle_media_mix);

		// Settings
		app.on_new_local_source(move || {
			let source = source::new_local_source();

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
					println!("files read correctly {:?}", records);
					database::add_records(records);
				},
				None => println!("Didn't receive a path. Result should be None: {:?}", source)
			}
		});
	}

	mod audio_control_events {
		pub fn handle_media_change(index: i32) {
			let move_to_next_audio = index > 0;
			println!("move_to_next_audio bool value: {}", move_to_next_audio);
		}

		pub fn handle_media_start(start: bool) {
			println!("start bool value: {}", start);
		}
		
		pub fn handle_media_loop(create_loop: bool) {
			println!("create_loop bool value: {}", create_loop);
		}

		pub fn handle_media_mix() {}
	}
}

// Note to self: Source and Database shouldn't interact with each other. Instead create functions here
// that utilize some functionality of source to call a function in database or vice versa.

pub fn validate_sources() -> Result<HashMap<String, MediaFile>, ErrorHandler> {
	let source_hashmap = database::get_table::<Source>()?;
	source::validate_sources(source_hashmap)
}
