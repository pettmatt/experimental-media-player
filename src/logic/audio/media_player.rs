use crate::logic::data_types::track::Track;
use crate::State;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::FromFileError;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

pub struct MediaPlayer {
    pub currently: Option<Rc<RefCell<Track>>>,
    queue: Vec<Rc<RefCell<Track>>>,
    controls: Controls,
    manager: Option<AudioManager<DefaultBackend>>,
    handle: Option<StreamingSoundHandle<kira::sound::FromFileError>>,
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
    pub fn new() -> Self {
        Self {
            currently: None,
            queue: Vec::new(),
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
        }
    }

    pub async fn play(&mut self, source: Rc<RefCell<Track>>) -> Result<(), Box<dyn std::error::Error>> {
	    println!("Should start to play with {:?}", &source);
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
	    let sound_data = StreamingSoundData::from_file(&path)?;
	    self.controls.length = Mutex::new(self.get_track_length(&sound_data));

	    let manager = self
	        .manager
	        .get_or_insert_with(|| AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
	        .expect("failed to create audio manager"));

	    let mut handle = manager.play(sound_data)?;
	    handle.set_volume(kira::Decibels(-20.0), Tween::default());

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
        self.queue = Vec::new();
        self.controls.position = Mutex::new(Duration::new(0, 0));
        self.manager = None;
        self.handle = None;
    }

    pub fn volume(&self) -> f32 {
        *self.controls.volume.lock().unwrap()
    }

    pub fn set_volume(&self, value: f32) {
        *self.controls.volume.lock().unwrap() = value;
    }

    // Checks if the track (item) is in queue and returns false if it is.
    // This can be used to prevent user from adding multiple same tracks.
    pub fn add(&mut self, track: Rc<RefCell<Track>>, i_know: bool) -> bool {
        if i_know == false {
            for item in self.queue.iter() {
                if item == &track {
                    return false;
                }
            }
        }
        self.queue.push(track);
        true
    }

    pub fn next(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.queue.len() > 0 {
            let track: Rc<RefCell<Track>> = self.queue.remove(0);
           	self.play(track);
            // if let Some(manager) = self.manager.as_mut() {
            //     self.currently = Some(track);
            //     if let Some(current) = &self.currently {
            //         let data = StreamingSoundData::from_file(current.path.clone())?;
            //         self.handle = Some(manager.play(data)?);
            //     }
            // }
        }
        Ok(())
    }

    pub fn previous(&mut self, state: &mut State) {}

    pub fn clear_queue(&self) {}

    fn load_queue_from_state(&mut self, state: &State) {
        // self.queue = state.queue.clone();

        // for item in self.queue.iter() {
        // 	if let Some(sink) = &self.sink {
        // 		self.add_to_queue(&state, media_file);
        // 	}
        // }
    }

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
