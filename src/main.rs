// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use logic::{database::{self, MediaFile, QueueItem}, ui};

mod logic;

slint::include_modules!();

#[derive(Clone, Debug, Default)]
struct State {
	index: Vec<MediaFile>,
// 	playing: Result<None, fmt::Error>,
	queue: Vec<QueueItem>,
// 	playlists: Vec<String>,
// 	sources: Vec<String>,
// 	settings: Settings
}

fn main() -> Result<(), Box<dyn Error>> {
	let app = AppWindow::new()?;
	let mut state = State::default();

	ui::hanle_initialization(&app, &mut state);
	ui::handle_events(&app, &mut state);

    app.run()?;

    Ok(())
}
