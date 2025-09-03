// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, error::Error};
use logic::{database::{self, MediaFile}, ui};

mod logic;

slint::include_modules!();

// struct Settings {}

#[derive(Debug)]
struct State {
	index: HashMap<String, MediaFile>,
// 	playing: Result<None, fmt::Error>,
// 	queue: Vec<String>,
// 	playlists: Vec<String>,
// 	sources: Vec<String>,
// 	settings: Settings
}

fn main() -> Result<(), Box<dyn Error>> {
	let mut state: State;
	
	{ // Initialization & recover last state
		if database::initialize_tables().is_ok() {
			let mut sIndex: HashMap<String, MediaFile> = HashMap::new();

			println!("Database initialized");
			if let Ok(index_hashmap) = database::get_table::<MediaFile>() {
				sIndex.extend(index_hashmap);
				println!("Fetched most recent details: {:?}", sIndex);
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
		let media_hashmap = database::get_table::<MediaFile>()?;
		println!("Files: {:?}", media_hashmap);
	}

    app.run()?;

    Ok(())
}
