// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{borrow::Cow, error::Error, fmt, i32};
use logic::{managment::{database, source::{get_local_files, MediaFile}}, ui};

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
			let table = Cow::from("main");
			let result = database::get_table::<MediaFile>(table);
		} else {
			println!("Couldn't create db connection for initialization")
		}
	}

	let app = AppWindow::new()?;

	ui::handle_events(&app);

	{ // Update the state, incase something has chagned
		let local_files = get_local_files();
		database::add_records(Cow::from("main"), local_files);
	}

    app.run()?;

    Ok(())
}
