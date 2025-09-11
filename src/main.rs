// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use logic::{database::{MediaFile, QueueItem}, ui};
use slint::ComponentHandle;

mod logic;

slint::include_modules!();

#[derive(Clone, Debug, Default)]
struct State {
	index: Vec<MediaFile>,
	queue: Vec<QueueItem>,
	// 	playing: Result<None, fmt::Error>,
	// 	playlists: Vec<String>,
	// 	sources: Vec<String>,
	// 	settings: Settings
}

fn main() -> Result<(), Box<dyn Error>> {
	let app = AppWindow::new()?;
	let mut state = State::default();

	ui::handle_initialization(&mut state);
	ui::handle_passing_values(&app, &state);
	ui::handle_events(&app, &state);

    app.run()?;

    Ok(())
}
