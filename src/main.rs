// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use logic::{database::{self, MediaFile, QueueItem}, ui};

mod logic;

slint::include_modules!();

// struct Settings {}

#[derive(Debug)]
struct State {
	index: Vec<MediaFile>,
// 	playing: Result<None, fmt::Error>,
	queue: Vec<QueueItem>,
// 	playlists: Vec<String>,
// 	sources: Vec<String>,
// 	settings: Settings
}

fn main() -> Result<(), Box<dyn Error>> {
	let state: State;
	
	{ // Initialization & recover last state
		if database::initialize_tables().is_ok() {
			let mut sIndex: Vec<MediaFile> = Vec::new();

			println!("Database initialized");
			if let Ok(list) = database::get_table::<MediaFile>() {
				sIndex.extend(list);
				println!("Fetched most recent details: {:?}", sIndex);
			}

			state = State {
				index: sIndex,
				queue: Vec::new()
			}
		} else {
			println!("Couldn't create db connection for initialization")
		}
	}

	let app = AppWindow::new()?;

	ui::handle_events(&app);

	{ // Update the state, incase something has chagned
		let read_sources = logic::validate_sources()?;
		println!("Checked files {:?}", &read_sources);
		database::add_records(read_sources);
		println!("Updated file sources");
		let media_list = database::get_table::<MediaFile>()?;
		println!("Files: {:?}", media_list);
	}

    app.run()?;

    Ok(())
}
