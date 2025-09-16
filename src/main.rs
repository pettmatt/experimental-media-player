// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use logic::{database::{MediaFile, QueueItem}, ui};
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

impl State {
	fn add_to_queue(&mut self, media: &MediaFile) {
		let mut item = QueueItem {
			media_id: media.id,
			currently_playing: false,
		};

		if self.queue.is_empty() {
			item.currently_playing = true;
		}

		self.queue.push(item);
	}

	fn remove_from_queue(&mut self, id: i32) {
		let found = self.queue.iter().position(|item| item.media_id == id);

		if let Some(index) = found {
			self.queue.remove(index);
		}
	}

	fn remove_first_from_queue(&mut self) {
		self.queue.remove(0);
		let first_record = &mut self.queue[0];
		first_record.currently_playing = true;
		// self.queue[0] = first_record.clone();
	}

	fn move_to_first_in_queue(&mut self, index: i32) {
		println!("Queue pre {:?}", self.queue);
		let item = self.queue.remove(index as usize);
		self.queue.insert(0, item);
		println!("Queue aft {:?}", self.queue);
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let app = AppWindow::new()?;
	let mut state = State::default();

	ui::handle_initialization(&mut state);
	ui::handle_passing_values(&app, &state);
	ui::handle_events(&app, &mut state);

    app.run()?;

    Ok(())
}
