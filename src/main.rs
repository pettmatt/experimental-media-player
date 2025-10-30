// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{cell::RefCell, error::Error, rc::Rc, time::{UNIX_EPOCH}};
use logic::{database::{MediaFile, QueueItem, Playlist, AudioEntry}, ui_events as ui};
use slint::{ComponentHandle, ModelRc, SharedString};

mod logic;

slint::include_modules!();

#[derive(Clone, Debug, Default)]
struct TimeLine {
	current: i32,
	length: i32,
	media_index: Option<usize>,
	queue_index: Option<usize>,
}

#[derive(Clone, Debug, Default)]
struct State {
	index: Vec<MediaFile>,
	queue: Vec<QueueItem>, // Because Rodio doesn't offer frexible way to interact with the queue, we're managing by deleting the queue, whenever we want to make a change.
	playing: TimeLine,
	playlists: Vec<Playlist>,
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
		println!("(State) Index state updated. {:?}", self.index);
	}

	pub fn set_queue(&mut self, queue: Option<Vec<QueueItem>>, globals: &SlintState) {
		if let Some(q) = queue {
			self.queue = q;
		}

		let media_queue = self.convert_queue();
		println!("(State) Queue state updated. {:?} ::: {:?}", self.queue, media_queue);
		globals.set_queue(ModelRc::from(&media_queue[..]));
	}

	fn set_new_playlist(&mut self, name: String, image_url: String, globals: &SlintState) {
		let new_entry = Playlist {
			id: 0,
			name,
			sources: Vec::new(),
			image_url,
			created_at: "".to_string(),
			listened_at: "".to_string(),
			audio_list: Vec::new(),
			_audio_list_string: String::new(),
			_sources_string: String::new(),
		};

		self.playlists.push(new_entry);
		let playlists: Vec<SlintPlaylist> = self.convert_playlist_to_slint();
		globals.set_playlist(ModelRc::from(&playlists[..]));
	}

	fn add_to_playlist(&mut self, playlist_id: i32, media_id: i32, globals: &SlintState) {
		let playlist: Option<&mut Playlist> = self.playlists.iter_mut().find(|playlist|
			playlist.id == playlist_id);

		if let Some(p) = playlist {
			let now = std::time::SystemTime::now();
			let since = now.duration_since(UNIX_EPOCH);

			if let Ok(duration) = since {
				let new_entry = AudioEntry {
					id: media_id,
					added_at: format!("{}", duration.as_secs()),
				};
				p.audio_list.push(new_entry);
			}

			let playlists: Vec<SlintPlaylist> = self.convert_playlist_to_slint();
			globals.set_playlist(ModelRc::from(&playlists[..]));
		};
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
				let media_file = self.find_source_by_id(q.media_id);
				if let Some((_, m)) = media_file {
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

	pub fn convert_playlist_to_slint(&self) -> Vec<slint_generatedAppWindow::SlintPlaylist> {
		self.playlists
			.clone()
			.into_iter()
			.map(|p| {
				slint_generatedAppWindow::SlintPlaylist {
					id: p.id,
					name: SharedString::from(p.name),
					sources: convert_to_slint_model(p.sources),
					image_url: SharedString::from(p.image_url),
					created_at: SharedString::from(p.created_at),
					listened_at: SharedString::from(p.listened_at),
					audio_list: convert_to_slint_model(p.audio_list),
				}
			})
			.collect()
	}

	pub fn find_source_by_id(&self, id: i32) -> Option<(usize, &MediaFile)> {
		self.index.iter().enumerate().find(|(_, media)| media.id == id)
	}

	pub fn merge_to_index(&mut self, records: Vec<MediaFile>) {
		let merged = self.index.clone();
    	self.index.extend(records.into_iter().filter(|item| !merged.contains(item)));
	}

	fn set_index_playing(&mut self, index: usize, value: bool) -> Option<()> {
		if let Some(queue_item) = self.queue.get(index) {
			if let Some((media_index, _)) = self.find_source_by_id(queue_item.media_id) {
				self.index[media_index].playing = value;

				if value {
					self.playing.media_index = Some(media_index);
					self.playing.queue_index = Some(index);
				}

				return Some(());
			}
		}

		None
	}
}

impl From<AudioEntry> for (SharedString, i32) {
    fn from(entry: AudioEntry) -> Self {
        (SharedString::from(entry.added_at), entry.id)
    }
}

fn convert_to_slint_model<T, E>(vec: Vec<T>) -> ModelRc<E>
where
	E: From<T> + Clone + 'static 
{
	let slint_vec: Vec<E> = vec.into_iter().map(|item| E::from(item)).collect();
    ModelRc::new(slint::VecModel::from(slint_vec))
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