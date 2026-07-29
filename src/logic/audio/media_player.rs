use crate::logic::data_types::track::Track;
use crate::State;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle};
use kira::sound::FromFileError;
use kira::{AudioManager, AudioManagerSettings, DefaultBackend, Tween};
use std::sync::Mutex;
use std::time::Duration;

pub struct MediaPlayer<'a> {
    currently: Option<&'a Track>,
    queue: Vec<&'a Track>,
    controls: Controls,
    manager: Option<AudioManager<DefaultBackend>>,
    handle: Option<StreamingSoundHandle<kira::sound::FromFileError>>,
}

struct Controls {
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

impl<'a> MediaPlayer<'a> {
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

    pub async fn play(&mut self, source: &'a Track) -> Result<(), Box<dyn std::error::Error>> {
        if self.controls.pause == false {
            return Ok(());
        }
        self.controls.pause = false;
        if self.currently.is_some() {
            if let Some(current) = self.currently {
                if current != source {
                    self.currently = Some(source);
                    self.queue.push(source);
                }
                if self.manager.is_some() {
                    if let Some(manager) = self.manager.as_mut() {
                        let sound_data =
                            StreamingSoundData::from_file("./requests/output/audio.mp3")?;
                        let handle = manager.play(sound_data)?;
                        self.handle = Some(handle);
                    }
                } else {
                    let mut manager =
                        AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
                    let sound_data = StreamingSoundData::from_file("./requests/output/audio.mp3")?;
                    self.controls.length = Mutex::new(self.get_track_length(&sound_data));
                    let mut handle = manager.play(sound_data)?;
                    handle.set_volume(kira::Decibels(-3.0), Tween::default());
                    self.manager = Some(manager);
                    self.handle = Some(handle);
                }
            }
        }
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

    pub fn is_loop(&self) -> bool {
        self.controls.loop_pick
    }

    pub fn is_rng(&self) -> bool {
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
    pub fn append(&mut self, track: &'a Track, i_know: bool) -> bool {
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
            let track: &Track = self.queue.remove(0);
            if let Some(manager) = self.manager.as_mut() {
                self.currently = Some(track);
                if let Some(current) = self.currently {
                    let data = StreamingSoundData::from_file(current.path.clone())?;
                    self.handle = Some(manager.play(data)?);
                }
            }
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
