use crate::logic::data_types::queue_item::QueueItem;
use crate::logic::data_types::track::Track;
use crate::State;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::FromFileError;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};
use std::cell::RefCell;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

pub struct MediaPlayer {
	pub global_state: Rc<RefCell<State>>,
	pub generation: u64,
    pub currently: Option<Rc<RefCell<Track>>>,
    controls: Controls,
    manager: Option<AudioManager<DefaultBackend>>,
    pub handle: Option<StreamingSoundHandle<kira::sound::FromFileError>>,
    pub end_timer: Option<slint::Timer>,
}

pub struct Controls {
    loop_pick: bool,
    rng_pick: bool,
    pause: bool,
    stopped: bool,
    volume: Mutex<f32>,
    position: Mutex<Duration>,
    length: Mutex<Duration>,
}

// TODO: Add a logic that loops the queue if wanted.
// TODO: Add a logic that puts tracks from queue to "trash"-queue.
// TODO: Add a logic picks random track (would use trash-queue to prevent to pick previous tracks).

impl MediaPlayer {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
    	let mut hasher = DefaultHasher::new();
     	5678.hash(&mut hasher);
        Self {
        	global_state: state,
            currently: None,
            manager: None,
            handle: None,
            controls: Controls {
                loop_pick: false,
                rng_pick: false,
                pause: false,
                stopped: false,
                volume: Mutex::new(1.0),
                position: Mutex::new(Duration::new(0, 0)),
                length: Mutex::new(Duration::new(0, 0)),
            },
       		generation: hasher.finish(),
         	end_timer: None
        }
    }

    pub async fn play(&mut self, source: Rc<RefCell<Track>>) -> Result<(), Box<dyn std::error::Error>> {
	    println!("Should start to play with {:?}", &source);
		self.global_state.borrow_mut().index_playing_reset();
	    self.controls.pause = false;

	    // Stop whatever's currently playing, if anything.
		// Play shouldn't check if it should play, it should just play.
		// The logic to check if something should play should be checked whatever is calling play().
	    if let Some(handle) = self.handle.as_mut() {
	        handle.stop(Tween::default());
	    }

	    self.currently = Some(source.clone());

	    let path = {
	        let track = source.borrow();
	        std::path::PathBuf::from(&track.path)
	    };

		source.borrow_mut().playing = true;
	    let sound_data = StreamingSoundData::from_file(&path)?;
	    self.controls.length = Mutex::new(self.get_track_length(&sound_data));

	    let manager = self
	        .manager
	        .get_or_insert_with(|| AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
	        .expect("failed to create audio manager"));

	    let mut handle = manager.play(sound_data)?;
		let volume = normalize_volume(self.global_state.borrow().volume, 0.0, 100.0, -60.0, 0.0); // Was -20.0
	    handle.set_volume(kira::Decibels(volume), Tween::default());

	    self.handle = Some(handle);
	    Ok(())
    }

    pub fn unpause(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            self.controls.pause = false;
            handle.resume(Tween::default());
        }
    }

    pub fn pause(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            self.controls.pause = true;
            handle.pause(Tween::default());
        }
    }

    pub fn is_paused(&self) -> bool {
        self.controls.pause
    }

    pub fn is_looped(&self) -> bool {
        self.controls.loop_pick
    }

    pub fn is_rnged(&self) -> bool {
        self.controls.rng_pick
    }

    pub fn set_queue_loop(&mut self, b: bool) {
        self.controls.loop_pick = b;
        self.controls.rng_pick = false;
    }

    pub fn set_queue_rng(&mut self, b: bool) {
        self.controls.rng_pick = b;
        self.controls.loop_pick = false;
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.as_mut() {
            handle.stop(Tween::default());
        }
    }

    pub fn play_specific_in_queue(&mut self, index: i32) {}

    pub fn get_track_length(&self, track_data: &StreamingSoundData<FromFileError>) -> Duration {
        track_data.duration()
    }

    pub fn clear(&mut self) {
        self.currently = None;
        self.controls.position = Mutex::new(Duration::new(0, 0));
        self.manager = None;
        self.handle = None;
    }

    pub fn volume(&self) -> f32 {
        *self.controls.volume.lock().unwrap()
    }

    pub fn set_volume(&mut self, value: f32) {
    	if let Some(handle) = self.handle.as_mut() {
     		let volume = normalize_volume(value, 0.0, 100.0, -60.0, 10.0);
    		handle.set_volume(kira::Decibels(volume), Tween::default());
     	}
        *self.controls.volume.lock().unwrap() = value;
    }

    pub fn next(&mut self) -> Result<Option<Rc<RefCell<Track>>>, Box<dyn std::error::Error>> {
        if self.global_state.borrow().queue.len() > 1 {
            let item: QueueItem = self.global_state.borrow_mut().queue.remove(0);
            let found = self.global_state.borrow().find_source_by_id(item.track_id);
            if let Some((_, track)) = found {
          		return Ok(Some(track));
            } else {
            	println!("Something went wrong, couldn't find next track {}", item.track_id);
            }
        }
        Ok(None)
    }

    pub fn previous(&mut self) -> Result<Option<Rc<RefCell<Track>>>, Box<dyn std::error::Error>>  {
	    if self.global_state.borrow().trash_queue.len() > 1 {
			if let Some(item) = self.global_state.borrow().trash_queue.last() {
		        let found = self.global_state.borrow().find_source_by_id(item.track_id);
		        if let Some((_, track)) = found {
		      		return Ok(Some(track));
		        } else {
		        	println!("Something went wrong, couldn't find previous track {}", item.track_id);
		        }
			}
	    }
	    Ok(None)
    }

    // fn load_queue_from_state(&mut self, state: &State) {
        // self.queue = state.queue.clone();

        // for item in self.queue.iter() {
        // 	if let Some(sink) = &self.sink {
        // 		self.add_to_queue(&state, media_file);
        // 	}
        // }
    // }

    pub fn try_seek(&mut self, position: f64) {
        if let Some(handle) = self.handle.as_mut() {
            handle.seek_to(position);
        }
    }

    pub fn try_seek_by(&mut self, amount: f64) {
        if let Some(handle) = self.handle.as_mut() {
            handle.seek_by(amount);
        }
    }

    pub fn get_position(&mut self) -> f64 {
        if let Some(handle) = self.handle.as_mut() {
            return handle.position();
        }

        0.0
    }

    pub async fn callback_after_audio_ends(&self, callback: fn()) {}
}

fn normalize_volume(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
	out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
}
