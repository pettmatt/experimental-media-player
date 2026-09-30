// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::logic::{
    data_types::{
        playlist::{AudioEntry, Playlist},
        queue_item::QueueItem,
        track::Track,
    },
    slint::convert_to_slint_model,
};
use logic::ui_events as ui;
use slint::{ComponentHandle, Model, ModelExt, ModelRc, SharedString};
use std::{cell::RefCell, error::Error, rc::Rc};
mod logic;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;
    let mut state = State::default();
    let tokio_runtime = tokio::runtime::Runtime::new()?;

    ui::handle_initialization(&mut state);
    ui::handle_passing_values(&app, &mut state);
    ui::handle_events(&app, &mut Rc::new(RefCell::new(state)), &tokio_runtime);
    app.run()?;

    Ok(())
}

#[derive(Clone, Debug, Default)]
struct TimeLine {
    current: i32,
    length: i32,
    media_index: Option<usize>,
    queue_index: Option<usize>,
}

impl TimeLine {
	pub fn set_timeline(&mut self, track: Rc<RefCell<Track>>, globals: SlintState) {
        self.length = track.borrow().duration;
        self.current = 0;
        self.media_index = Some(track.borrow().id as usize);
        let timeline: SlintTimeline = self.convert_timeline_to_slint();
        globals.set_timeline(timeline);
    }

    pub fn update_timeline(&mut self, value: i32, globals: SlintState) {
        self.current = value;
        let timeline: SlintTimeline = self.convert_timeline_to_slint();
        globals.set_timeline(timeline);
    }

    fn convert_timeline_to_slint(&self) -> slint_generatedAppWindow::SlintTimeline {
	    slint_generatedAppWindow::SlintTimeline {
			current: self.current,
			length: self.length,
	        str_current: SharedString::from(format_into_time(self.current as f64)),
	        str_length: SharedString::from(format_into_time(self.length as f64)),
	    }
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    index: Vec<Rc<RefCell<Track>>>,
    queue: Vec<QueueItem>,
    trash_queue: Vec<QueueItem>,
    timeline: TimeLine,
    playlists: Vec<Playlist>,
    // 	sources: Vec<String>,
    // 	settings: Settings,
    volume: f32,
    search_result: Vec<Track>,
}

impl State {
 	pub fn set_state(&mut self, globals: &SlintState) {
  		let current_track = self.index.iter().find(|track| {
	  		track.borrow().playing
	  	}).unwrap();

    	globals.set_index(ModelRc::from(&self.convert_index()[..]));
     	globals.set_queue(ModelRc::from(&self.convert_queue()[..]));
      	globals.set_playlist(ModelRc::from(&self.convert_playlist()[..]));
       	globals.set_timeline(self.convert_timeline());
        // globals.set_settings(state.get_settings());
        globals.set_current_track(self.convert_track(&current_track.borrow()));
        globals.set_volume(self.volume.clone());
        globals.set_search_result(ModelRc::from(&self.convert_search_tracks()[..]));
    }

	// Sets slint index by converting application's state to Slint format
    pub fn set_index(&mut self, index: Option<Vec<Rc<RefCell<Track>>>>, globals: &SlintState) {
        if let Some(i) = index {
            self.index = i;
        }

        let index: Vec<SlintTrack> = self.convert_index();
        globals.set_index(ModelRc::from(&index[..]));
    }

    // Controls how tracks are added to the system.
    // For example when searching tracks potentially from online, we potentially need to add
    // tracks on the fly to the index and/or a playlist without disturbing the user.
    pub fn add_to_index() {}

    pub fn set_queue(&mut self, queue: Option<Vec<QueueItem>>, globals: &SlintState) {
        if let Some(q) = queue {
            self.queue = q;
        }

        let media_queue = self.convert_queue();
        println!(
            "(State) Queue state updated. {:?} ::: {:?}",
            self.queue, media_queue
        );
        globals.set_queue(ModelRc::from(&media_queue[..]));
    }

   	pub fn set_volume(&mut self, globals: &SlintState) {
        globals.set_volume(self.volume);
    }

    pub fn set_current_track(&mut self, globals: &SlintState) {
  		if let Some(track) = self.convert_next_track() {
       		globals.set_current_track(track);
        	println!("globals {:?}", globals.get_current_track());
    	}
    }

    fn set_playlist(&mut self, globals: &SlintState) {
        let playlists: Vec<SlintPlaylist> = self.convert_playlist();
        globals.set_playlist(ModelRc::from(&playlists[..]));
    }

    fn set_search_result(&mut self, tracks: Vec<Track>, globals: &SlintState) {
      	self.search_result = tracks;
    	let converted_tracks: Vec<slint_generatedAppWindow::SlintTrack> =
     		self.search_result.iter().map(|t| self.convert_track(t)).collect();
     	let model = slint::ModelRc::from(Rc::new(slint::VecModel::from(converted_tracks)));

    	globals.set_search_result(model);

    	for track in globals.get_search_result().iter() {
   			println!("slint search result {:?}", track);
    	}
    }

    fn index_playing_reset(&mut self) {
    	for t in self.index.iter_mut() {
     		let mut track = t.borrow_mut();
     		if track.playing {
       			track.playing = false;
       		}
     	}
    }

    fn add_to_queue(&mut self, track_id: i32, i_know: bool, globals: &SlintState) -> bool {
    	if i_know {
	    	if let Some(track) = self.index
	     		.iter()
	       		.find(|t| t.borrow().id == track_id)
	     	{
	        	self.queue.push(QueueItem { track_id: track.borrow().id });
	         	println!("Queue {:?}", self.queue);
	     	}

			let queue: Vec<SlintTrack> = self.convert_queue();
            globals.set_queue(ModelRc::from(&queue[..]));

			return true;
     	}

     	false
    }

    fn add_to_playlist(&mut self, playlist_id: i32, media_id: i32, globals: &SlintState) {
        let playlist: Option<&mut Playlist> = self
            .playlists
            .iter_mut()
            .find(|playlist| playlist.id == playlist_id);

        if let Some(p) = playlist {
            let now = std::time::SystemTime::now();
            let since = now.duration_since(std::time::UNIX_EPOCH);

            if let Ok(duration) = since {
                let new_entry = AudioEntry {
                    id: media_id,
                    added_at: format!("{}", duration.as_secs()),
                };

                if p.tracks.is_none() {
                    p.tracks = Some(Vec::new())
                }

                if let Some(tracks) = p.tracks.as_mut() {
                    tracks.push(new_entry);
                }
            }

            let playlists: Vec<SlintPlaylist> = self.convert_playlist();
            globals.set_playlist(ModelRc::from(&playlists[..]));
        };
    }

    pub fn convert_index(&self) -> Vec<slint_generatedAppWindow::SlintTrack> {
        self.index
            .clone()
            .into_iter()
            .map(|t| slint_generatedAppWindow::SlintTrack {
                id: t.borrow().id,
                artist: t.borrow().artist.clone().into(),
                title: t.borrow().title.clone().into(),
                path: t.borrow().path.clone().into(),
                genre: t.borrow().genre.clone().into(),
                year: t.borrow().year.to_string().into(),
                extension: t.borrow().extension.clone().into(),
                duration: t.borrow().duration,
                str_duration: SharedString::from(format_into_time(t.borrow().duration as f64)),
                file_size: t.borrow().file_size,
                thumbnail: t.borrow().thumbnail.clone().into(),
                playing: t.borrow().playing,
            })
            .collect()
    }

    pub fn convert_queue(&self) -> Vec<slint_generatedAppWindow::SlintTrack> {
        self.queue
            .clone()
            .into_iter()
            .map(|q| {
                let track = self.find_source_by_id(q.track_id);
                if let Some((_, t)) = track {
                    return slint_generatedAppWindow::SlintTrack {
                        id: t.borrow().id,
                        title: t.borrow().title.clone().into(),
                        artist: t.borrow().artist.clone().into(),
                        path: t.borrow().path.clone().into(),
                        genre: t.borrow().genre.clone().into(),
                        year: t.borrow().year.to_string().into(),
                        extension: t.borrow().extension.clone().into(),
                        file_size: t.borrow().file_size,
                        duration: t.borrow().duration,
                        str_duration: SharedString::from(format_into_time(t.borrow().duration as f64)),
                        thumbnail: t.borrow().thumbnail.clone().into(),
                        playing: t.borrow().playing,
                    };
                }

                slint_generatedAppWindow::SlintTrack {
                    id: i32::MAX,
                    title: SharedString::from(""),
                    artist: SharedString::from(""),
                    path: SharedString::from(""),
                    genre: SharedString::from(""),
                    year: SharedString::from(""),
                    extension: SharedString::from(""),
                    file_size: 0,
                    duration: 0,
                    str_duration: SharedString::from(""),
                    thumbnail: SharedString::from(""),
                    playing: false,
                }
            })
            .collect()
    }

    pub fn convert_next_track(&self) -> Option<slint_generatedAppWindow::SlintTrack> {
		if let Some(item) = self.queue.first() {
	  		if let Some((_, t)) = self.find_source_by_id(item.track_id) {
             		return Some(slint_generatedAppWindow::SlintTrack {
                       id: t.borrow().id,
                       title: t.borrow().title.clone().into(),
                       artist: t.borrow().artist.clone().into(),
                       path: t.borrow().path.clone().into(),
                       genre: t.borrow().genre.clone().into(),
                       year: t.borrow().year.clone().into(),
                       thumbnail: t.borrow().thumbnail.clone().into(),
                       extension: t.borrow().extension.clone().into(),
                       file_size: t.borrow().file_size,
                       duration: t.borrow().duration,
                       str_duration: SharedString::from(format_into_time(t.borrow().duration as f64)),
                       playing: t.borrow().playing,
                   });
	    	}
		}

        	None
	}

	pub fn convert_track(&self, track: &Track) -> slint_generatedAppWindow::SlintTrack {
  		slint_generatedAppWindow::SlintTrack {
            id: track.id,
            title: track.title.clone().into(),
            artist: track.artist.clone().into(),
            path: track.path.clone().into(),
            genre: track.genre.clone().into(),
            year: track.year.clone().into(),
            thumbnail: track.thumbnail.clone().into(),
            extension: track.extension.clone().into(),
            file_size: track.file_size,
            duration: track.duration,
            str_duration: SharedString::from(format_into_time(track.duration as f64)),
            playing: track.playing,
		}
	}

	pub fn convert_timeline(&self) -> slint_generatedAppWindow::SlintTimeline {
		slint_generatedAppWindow::SlintTimeline {
			current: self.timeline.current.clone(),
			length: self.timeline.length.clone(),
			str_current: self.timeline.current.to_string().into(),
			str_length: self.timeline.length.to_string().into(),
		}
	}

	pub fn convert_search_tracks(&self) -> Vec<slint_generatedAppWindow::SlintTrack> {
		self.search_result.iter().map(|t| {
			slint_generatedAppWindow::SlintTrack {
				id: t.id,
	            title: t.title.clone().into(),
	            artist: t.artist.clone().into(),
	            path: t.path.clone().into(),
	            genre: t.genre.clone().into(),
	            year: t.year.clone().into(),
				thumbnail: t.thumbnail.clone().into(),
	            extension: t.extension.clone().into(),
	            file_size: t.file_size,
	            duration: t.duration,
	            str_duration: SharedString::from(format_into_time(t.duration as f64)),
	            playing: t.playing,
			}
		}).collect()
	}

    pub fn convert_playlist(&self) -> Vec<slint_generatedAppWindow::SlintPlaylist> {
        self.playlists
            .clone()
            .into_iter()
            .map(|p| {
                let mut sources = Vec::new();
                let mut tracks = Vec::new();
                let mut artist = String::from("");

                if let Some(s) = p.sources {
                    sources = s;
                }

                if let Some(t) = p.tracks {
                    tracks = t;
                }

                if let Some(a) = p.artist {
                	artist = a
                }

                slint_generatedAppWindow::SlintPlaylist {
                    id: p.id,
                    name: SharedString::from(p.name),
                    artist: SharedString::from(artist),
                    list_type: SharedString::from(p.list_type),
                    thumbnail: SharedString::from(p.thumbnail),
                    created_at: SharedString::from(p.created_at),
                    listened_at: SharedString::from(p.listened_at),
                    sources: convert_to_slint_model(sources),
                    tracks: convert_to_slint_model(tracks),
                }
            })
            .collect()
    }

    pub fn find_source_by_id(&self, id: i32) -> Option<(usize, Rc<RefCell<Track>>)> {
        self.index
            .iter()
            .enumerate()
            .find(|(_, track)| track.borrow().id == id)
            .map(|(i, track)| (i, track.clone()))
    }

    pub fn merge_to_index(&mut self, records: Vec<Track>) {
        let new_entries: Vec<Rc<RefCell<Track>>> = records
            .into_iter()
            .filter(|record| {
                !self.index.iter().any(|existing| existing.borrow().id == record.id)
            })
            .map(|t| Rc::new(RefCell::new(t)))
           	.collect();

        self.index.extend(new_entries);
    }

    pub fn set_index_playing(&mut self, index: usize, value: bool) -> Option<()> {
        if let Some(queue_item) = self.queue.get(index) {
            if let Some((track_index, track)) = self.find_source_by_id(queue_item.track_id) {
                track.borrow_mut().playing = value;

                if value {
                    self.timeline.media_index = Some(track_index);
                    self.timeline.queue_index = Some(index);
                }

                return Some(());
            }
        }

        None
    }
}

fn format_into_time(seconds: f64) -> String {
	let total = seconds.max(0.0).round() as u64;
 	let hours = total / 3600;
  	let minutes = (total % 3600) / 60;
   	let secs = total % 60;

    if hours > 0 {
    	format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}
