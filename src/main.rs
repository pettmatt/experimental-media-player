// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, rc::Rc};
use logic::{database::{MediaFile, QueueItem}, ui_events as ui};
use slint::{ComponentHandle, ModelRc, SharedString};

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
	// 	settings: Settings,
}

fn main() -> Result<(), Box<dyn Error>> {
	let app = AppWindow::new()?;
	let mut state = State::default();

	ui::handle_initialization(&mut state);
	ui::handle_passing_values(&app, &mut state);
	ui::handle_events(&app, &mut Rc::new(RefCell::new(state)));

    app.run()?;

    Ok(())
}

impl State {
	pub fn set_index(&mut self, index: Option<Vec<MediaFile>>, globals: &SlintState) {
		if let Some(i) = index {
			self.index = i;
		}

		let media_items = self.convert_index();
		globals.set_index(ModelRc::from(&media_items[..]));
		println!("(State) Index state updated.");
	}

	pub fn set_queue(&mut self, queue: Option<Vec<QueueItem>>, globals: &SlintState) {
		if let Some(q) = queue {
			self.queue = q;
		}

		let media_queue = self.convert_queue();
		globals.set_queue(ModelRc::from(&media_queue[..]));
		println!("(State) Queue state updated.");
	}

	pub fn convert_index(&self) -> Vec<slint_generatedAppWindow::SlintMediaFile> {
		self.index.clone()
			.into_iter()
			.map(|m| slint_generatedAppWindow::SlintMediaFile {
				id: m.id,
				artist: m.artist.into(),
				extension: m.extension.into(),
				name: m.name.into(),
				path: m.path.into(),
				duration: m.duration,
				file_size: m.file_size,
				playing: m.playing,
			})
			.collect()
	}

	pub fn convert_queue(&self) -> Vec<slint_generatedAppWindow::SlintMediaFile> {
		self.queue.clone()
			.into_iter()
			.map(|q| {
				let media_file = self.find_source_for_queue_item(q.media_id);
				if let Some(m) = media_file {
					return slint_generatedAppWindow::SlintMediaFile {
						id: m.id,
						artist: m.artist.clone().into(),
						extension: m.extension.clone().into(),
						name: m.name.clone().into(),
						path: m.path.clone().into(),
						duration: m.duration,
						file_size: m.file_size,
						playing: m.playing,
					};
				}

				slint_generatedAppWindow::SlintMediaFile {
					id: i32::MAX,
					artist: SharedString::from(""),
					extension: SharedString::from(""),
					name: SharedString::from(""),
					path: SharedString::from(""),
					duration: 0,
					file_size: 0,
					playing: false,
				}
			})
			.collect()
	}

	pub fn find_source_for_queue_item(&self, id: i32) -> Option<&MediaFile> {
		self.index.iter().find(|media| media.id == id)
	}

	pub fn merge_to_index(&mut self, records: Vec<MediaFile>) {
		let merged = self.index.clone();
    	self.index.extend(records.into_iter().filter(|item| !merged.contains(item)));
	}
}

impl TimeLine {
	pub fn update_timeline(&mut self, track: &MediaFile) {
		self.length = track.duration;
		self.current = 0;
	}

	pub fn update_position(&mut self, value: i32) {
		self.current = value;
	}
}