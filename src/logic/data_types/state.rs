use crate::{SlintPlaylist, SlintState, SlintTrack, logic::{convert::{format_into_time, to_model_rc}, data_types::{SlintConvertable, ToSlint, artist::Artist, playlist::{AudioEntry, Playlist}, queue_item::QueueItem, timeline::Timeline, track::Track}}, slint_generatedAppWindow};
use std::{cell::RefCell, rc::Rc};
use slint::{ModelRc, SharedString, VecModel};

#[derive(Clone, Debug, Default)]
pub struct SearchResults {
	pub tracks: Vec<Track>,
	pub artists: Vec<Artist>,
	pub playlists: Vec<Playlist>,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub index: Vec<Rc<RefCell<Track>>>,
    pub queue: Vec<QueueItem>,
    pub trash_queue: Vec<QueueItem>,
    pub timeline: Timeline,
    pub playlists: Vec<Playlist>,
    pub artists: Vec<Artist>,
    // 	sources: Vec<String>,
    // 	settings: Settings,
    pub volume: f32,
    pub search_result: SearchResults,
}

impl State {
 	pub fn set_state(&mut self, globals: &SlintState) {
  		let current_track = self.index.iter().find(|track| {
	  		track.borrow().playing
	  	});

    	globals.set_index(ModelRc::from(&self.index_to_slint()[..]));
     	globals.set_queue(ModelRc::from(&self.queue_to_slint()[..]));
      	globals.set_playlist(ModelRc::from(&self.convert_playlists()[..]));
       	globals.set_timeline(self.timeline.to_slint(None));
        // globals.set_settings(state.get_settings());
        if let Some(track) = current_track {
        	globals.set_current_track(track.borrow().to_slint(None));
        }
        globals.set_volume(self.volume.clone());
    }

	// Sets slint index by converting application's state to Slint format
    pub fn set_index(&mut self, index: Option<Vec<Rc<RefCell<Track>>>>, globals: &SlintState) {
        if let Some(i) = index {
            self.index = i;
        }

        let index: Vec<SlintTrack> = self.index_to_slint();
        globals.set_index(ModelRc::from(&index[..]));
    }

    // Controls how tracks are added to the system.
    // For example when searching tracks potentially from online, we potentially need to add
    // tracks on the fly to the index and/or a playlist without disturbing the user.
    pub fn add_to_index() {}

    pub fn set_main_view(&mut self, value: crate::SlintSelected, globals: &SlintState) {
  		globals.set_selected_item(value);
    }

    pub fn set_queue(&mut self, queue: Option<Vec<QueueItem>>, globals: &SlintState) {
        if let Some(q) = queue {
            self.queue = q;
        }

        let media_queue = self.queue_to_slint();
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

    pub fn set_playlist(&mut self, globals: &SlintState) {
        let playlists: Vec<SlintPlaylist> = self.convert_playlists();
        globals.set_playlist(ModelRc::from(&playlists[..]));
    }

    pub fn set_search_results(&mut self, search_results: SearchResults, globals: &SlintState) {
      	self.search_result = search_results;

    	let converted_tracks: Vec<slint_generatedAppWindow::SlintTrack> =
     		self.search_result.tracks
       			.iter().map(|t| t.to_slint(None)).collect();
     	let converted_artists: Vec<slint_generatedAppWindow::SlintArtist> =
      		self.search_result.artists
        		.iter().map(|a| a.to_slint(self.index.clone())).collect();
      	let converted_playlists: Vec<slint_generatedAppWindow::SlintPlaylist> =
      		self.search_result.playlists
        		.iter().map(|p| p.to_slint(None)).collect();

      	let model = slint_generatedAppWindow::SlintSearchResults {
       		tracks: ModelRc::from(&converted_tracks[..]),
       		artists: ModelRc::from(&converted_artists[..]),
         	playlists: ModelRc::from(&converted_playlists[..]),
      	};

    	globals.set_search_results(model);
    }

    pub fn index_playing_reset(&mut self) {
    	for t in self.index.iter_mut() {
     		let mut track = t.borrow_mut();
     		if track.playing {
       			track.playing = false;
       		}
     	}
    }

    pub fn add_to_queue(&mut self, track_id: i32, i_know: bool, globals: &SlintState) -> bool {
    	if i_know {
	    	if let Some(track) = self.index
	     		.iter()
	       		.find(|t| t.borrow().id == track_id)
	     	{
	        	self.queue.push(QueueItem { track_id: track.borrow().id });
	         	println!("Queue {:?}", self.queue);
	     	}

			let queue: Vec<SlintTrack> = self.queue_to_slint();
            globals.set_queue(ModelRc::from(&queue[..]));

			return true;
     	}

     	false
    }

    pub fn add_to_playlist(&mut self, playlist_id: i32, track_id: i32, globals: &SlintState) {
        let playlist: Option<&mut Playlist> = self
            .playlists
            .iter_mut()
            .find(|playlist| playlist.id == playlist_id);

        if let Some(p) = playlist {
            let now = std::time::SystemTime::now();
            let since = now.duration_since(std::time::UNIX_EPOCH);

            if let Ok(duration) = since {
                let new_entry = AudioEntry {
                    id: track_id,
                    added_at: format!("{}", duration.as_secs()),
                };

                if p.tracks.is_none() {
                    p.tracks = Some(Vec::new())
                }

                if let Some(tracks) = p.tracks.as_mut() {
                    tracks.push(new_entry);
                }
            }

            let playlists: Vec<SlintPlaylist> = self.convert_playlists();
            globals.set_playlist(ModelRc::from(&playlists[..]));
        };
    }

    pub fn index_to_slint(&self) -> Vec<slint_generatedAppWindow::SlintTrack> {
        self.index
            .clone()
            .into_iter()
            .map(|t| {
     			let genres: VecModel<SharedString> = t.borrow().genres
        			.iter().map(|g| SharedString::from(g.as_str())).collect();
        		let genre_model: ModelRc<SharedString> = ModelRc::new(VecModel::from(genres));

            	slint_generatedAppWindow::SlintTrack {
	                id: t.borrow().id,
	                artist: t.borrow().artist.clone().into(),
	                title: t.borrow().title.clone().into(),
	                path: t.borrow().path.clone().into(),
	                genres: genre_model,
	                year: t.borrow().year.to_string().into(),
	                extension: t.borrow().extension.clone().into(),
	                duration: t.borrow().duration,
	                str_duration: SharedString::from(format_into_time(t.borrow().duration as f64)),
	                file_size: t.borrow().file_size,
	                thumbnail: t.borrow().create_image_from_path(),
	                playing: t.borrow().playing,
	            }
            })
            .collect()
    }

    pub fn queue_to_slint(&self) -> Vec<slint_generatedAppWindow::SlintTrack> {
        self.queue
            .clone()
            .into_iter()
            .map(|q| {
                let track = self.find_source_by_id(q.track_id);

                if let Some((_, t)) = track {
                    return t.borrow().to_slint(None)
                }

                Track::to_slint_empty()
            })
            .collect()
    }

    pub fn convert_next_track(&self) -> Option<slint_generatedAppWindow::SlintTrack> {
		if let Some(item) = self.queue.first() {
	  		if let Some((_, t)) = self.find_source_by_id(item.track_id) {
             		return Some(t.borrow().to_slint(None));
	    	}
		}

       	None
	}

	pub fn convert_search_results(&self) -> slint_generatedAppWindow::SlintSearchResults {
		let tracks = self.search_result.tracks.iter().map(|t| {
			t.to_slint(None)
		}).collect();


		let artists = self.search_result.artists.iter().map(|a| {
			a.to_slint(self.index.clone())
		}).collect();

		let playlists = self.search_result.playlists.clone().into_iter().map(|p| {
			p.to_slint(None)
		}).collect();

		slint_generatedAppWindow::SlintSearchResults {
			tracks: to_model_rc(tracks),
			artists: to_model_rc(artists),
			playlists: to_model_rc(playlists),
		}
	}

    pub fn convert_playlists(&self) -> Vec<slint_generatedAppWindow::SlintPlaylist> {
        self.playlists
            .clone()
            .into_iter()
            .map(|p| {
                p.to_slint(None)
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
