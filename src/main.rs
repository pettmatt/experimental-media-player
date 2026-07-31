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
use slint::{ComponentHandle, ModelRc, SharedString};
use std::{cell::RefCell, error::Error, rc::Rc};
mod logic;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;
    let mut state = State::default();

    ui::handle_initialization(&mut state);
    ui::handle_passing_values(&app, &mut state);
    ui::handle_events(&app, &mut Rc::new(RefCell::new(state)));

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
    pub fn update_timeline(&mut self, track: &Track) {
        self.length = track.duration;
        self.current = 0;
    }

    pub fn update_position(&mut self, value: i32) {
        self.current = value;
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    index: Vec<Rc<RefCell<Track>>>,
    queue: Vec<QueueItem>, // Because Rodio doesn't offer frexible way to interact with the queue, we're managing by deleting the queue, whenever we want to make a change.
    playing: TimeLine,
    playlists: Vec<Playlist>,
    // 	sources: Vec<String>,
    // 	settings: Settings,
}

impl State {
    pub fn set_index(&mut self, index: Option<Vec<Rc<RefCell<Track>>>>, globals: &SlintState) {
        if let Some(i) = index {
            self.index = i;
        }

        let index: Vec<SlintTrack> = self.convert_index();
        globals.set_index(ModelRc::from(&index[..]));
    }

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

    fn set_new_playlist(&mut self, globals: &SlintState) {
        let playlists: Vec<SlintPlaylist> = self.convert_playlist_to_slint();
        globals.set_playlist(ModelRc::from(&playlists[..]));
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

            let playlists: Vec<SlintPlaylist> = self.convert_playlist_to_slint();
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
                year: t.borrow().year as i32,
                extension: t.borrow().extension.clone().into(),
                duration: t.borrow().duration,
                file_size: t.borrow().file_size,
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
                        year: t.borrow().year as i32,
                        extension: t.borrow().extension.clone().into(),
                        file_size: t.borrow().file_size,
                        duration: t.borrow().duration,
                        playing: t.borrow().playing,
                    };
                }

                slint_generatedAppWindow::SlintTrack {
                    id: i32::MAX,
                    title: SharedString::from(""),
                    artist: SharedString::from(""),
                    path: SharedString::from(""),
                    genre: SharedString::from(""),
                    year: 0,
                    extension: SharedString::from(""),
                    file_size: 0,
                    duration: 0,
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
                let mut sources = Vec::new();
                let mut tracks = Vec::new();

                if let Some(s) = p.sources {
                    sources = s;
                }

                if let Some(t) = p.tracks {
                    tracks = t;
                }

                slint_generatedAppWindow::SlintPlaylist {
                    id: p.id,
                    name: SharedString::from(p.name),
                    image_url: SharedString::from(p.image_url),
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
                    self.playing.media_index = Some(track_index);
                    self.playing.queue_index = Some(index);
                }

                return Some(());
            }
        }

        None
    }
}
