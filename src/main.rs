// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use logic::{managment::{database, source::{validate_sources, MediaFile}}, ui};

mod logic;

slint::include_modules!();

struct Settings {}
struct Authentication {}

// struct State {
// 	index: 
// 	playing: Result<None, fmt::Error>,
// 	queue: Vec<String>,
// 	playlists: Vec<String>,
// 	sources: Vec<String>,
// 	authentications: Vec<Authentication>,
// 	settings: Settings
// }

fn main() -> Result<(), Box<dyn Error>> {
	// let state: State; // Will hold the main state of Slint
	
	{ // Initialization & recover last state
		if database::initialize_tables().is_ok() {
			println!("Database initialized");
			let media_hashmap = database::get_table::<MediaFile>()?;
			println!("Fetched most recent details: {:?}", media_hashmap);
		} else {
			println!("Couldn't create db connection for initialization")
		}
	}

	let app = AppWindow::new()?;

	ui::handle_events(&app);

	{ // Update the state, incase something has chagned
		let read_sources = validate_sources()?;
		println!("Checked files {:?}", &read_sources);
		database::add_records(read_sources);
		println!("Updated file sources");
		let media_hashmap = database::get_table::<MediaFile>()?;
		println!("Files: {:?}", media_hashmap);
	}

    app.run()?;

    Ok(())
}
