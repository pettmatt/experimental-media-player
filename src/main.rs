// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, rc::Rc};
use logic::{database::{MediaFile, QueueItem}, ui_events as ui};
use slint::ComponentHandle;

mod logic;

slint::include_modules!();

#[derive(Clone, Debug, Default)]
struct TimeLine {
	current: i32,
	length: i32,
}

#[derive(Clone, Debug, Default)]
struct State {
	index: Vec<MediaFile>,
	queue: Vec<QueueItem>, // Because Rodio doesn't offer frexible way to interact with the queue, we're managing by deleting the queue, whenever we want to make a change.
	playing: TimeLine,
	// 	playlists: Vec<String>,
	// 	sources: Vec<String>,
	// 	settings: Settings
}

fn main() -> Result<(), Box<dyn Error>> {
	let app = AppWindow::new()?;
	let mut state = State::default();

	ui::handle_initialization(&mut state);
	ui::handle_passing_values(&app, &state);
	ui::handle_events(&app, &mut Rc::new(RefCell::new(state)));

    app.run()?;

    Ok(())
}

impl State {
	pub fn find_source_for_queue_item(&self, id: i32) -> Option<&MediaFile> {
		self.index.iter().find(|media| media.id == id)
	}
}
