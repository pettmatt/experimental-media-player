use super::sink::Sink;
use crate::{logic::data_types::track::Track, State};
// use rodio::{self, Decoder, OutputStream};
use std::{
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

// Rodio docs: https://docs.rs/rodio/latest/rodio/
// Note: The sound plays in a separate audio thread,
// so we need to keep the main thread alive while it's playing.

pub struct MediaPlayer {
    sink: Option<Arc<Mutex<Sink>>>,
    // stream_handle: Option<OutputStream>,
}

impl MediaPlayer {
    pub fn new() -> Self {
        Self {
            sink: None,
            // stream_handle: None,
        }
    }

    pub fn start(&mut self, audio: &Track) {
        if self.sink.is_none() {
            // let (stream_handle, new_sink) = open_stream();
            // self.sink = Some(Arc::new(Mutex::new(new_sink)));
            // self.stream_handle = Some(stream_handle);
        }

        if let Some(guard) = &self.sink {
            let audio_path = Path::new(&audio.path);
            if let Ok(sink) = guard.lock() {
                sink.set_volume(0.1);
                // start_playing(&sink, audio_path);
            }
        }
    }

    pub fn start_next(&mut self, audio: &Track) {}

    pub fn source_toggle(&mut self) {
        if self.sink.is_none() {
            return;
        }

        if let Some(guard) = &self.sink {
            if let Ok(sink) = guard.lock() {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
        }
    }

    pub fn pause(&mut self) {
        if let Some(guard) = &self.sink {
            if let Ok(sink) = guard.lock() {
                if !sink.is_paused() {
                    sink.pause();
                }
            }
        }
    }

    pub fn source_change_to_specific(&mut self, index: i32) {
        if let Some(sink) = &mut self.sink {
            // *sink.detach();
        }
    }

    pub fn next(&self) {
        if let Some(guard) = &self.sink {
            if let Ok(sink) = guard.lock() {
                sink.skip_one();
            }
        }
    }

    pub fn previous(&mut self, state: &mut State) {
        if let Some(sink) = &self.sink {
            // let current_audio: Vec<QueueItem> = state.queue
            // 	.clone()
            // 	.into_iter()
            // 	.filter(|item| item.playing == false)
            // 	.collect();

            // let mut previous_audio = None;

            // let mut index: i32 = 0;
            // for audio in state.index.iter() {
            // 	if audio.id == current_audio[0].media_id {
            // 		break;
            // 	}

            // 	index += 1;
            // }

            // if let Some(audio) = previous_audio {
            // 	// Destroy current queue
            // 	self.destroy_sink();
            // 	self.new_sink();
            // 	self.add_to_queue(state, audio.clone()); // Add the previous track to queue.
            // 	sink.play();

            // 	self.load_queue_from_state(state);
            // }

            // sink.append(source);
            // update_playing(state, -1);
        }
    }

    pub fn clear_queue(&self) {
        if let Some(stream_handle) = &self.stream_handle {
            // stream_handle.cl
        }
    }

    pub fn create_queue(&self, media_files: Vec<&Track>) -> Result<(), ()> {
        let _ = media_files.iter().map(|media_file| {
            if let Some(sink) = &self.sink {
                {
                    let guard = sink.lock().unwrap();
                    guard.clear();
                }

                let file = File::open(&media_file.path).unwrap();
                // let source = Decoder::try_from(file).unwrap();

                if let Some(guard) = self.sink.as_ref() {
                    if let Ok(sink) = guard.lock() {
                        // sink.append(source);
                    }
                }
            }
        });

        Ok(())
    }

    pub fn add_to_queue(&self, state: &mut State, media_file: &Track) -> Result<(), ()> {
        if let Ok(file) = File::open(&media_file.path) {
            // if let Ok(source) = Decoder::try_from(file) {
            //     if let Some(guard) = &self.sink {
            //         if let Ok(sink) = guard.lock() {
            //             sink.append(source);
            //             // state.queue.push(QueueItem {
            //             // 	media_id: media_file.id,
            //             // 	playing: false,
            //             // });

            //             return Ok(());
            //         }
            //     }
            // } else {
            //     println!("Can't add media to queue. Couldn't convert file.");
            // }
        } else {
            println!(
                "Can't add media to queue. Path might be unvalid: {}",
                media_file.path
            );
        }

        Err(())
    }

    fn load_queue_from_state(&mut self, state: &State) {
        // self.queue = state.queue.clone();

        // for item in self.queue.iter() {
        // 	if let Some(sink) = &self.sink {
        // 		self.add_to_queue(&state, media_file);
        // 	}
        // }
    }

    pub fn change_position(&mut self, position: Duration) {
        if let Some(guard) = &self.sink {
            if let Ok(sink) = guard.lock() {
                let _ = sink.try_seek(position);
            }
        }
    }

    pub fn get_position(&self) -> u32 {
        if let Some(guard) = &self.sink {
            if let Ok(sink) = guard.lock() {
                return sink.get_pos().as_secs_f32() as u32;
            }
        }

        0
    }

    pub fn set_volume(&self, value: f32) {
        if let Some(sink) = &self.sink {
            if let Ok(guard) = sink.lock() {
                guard.set_volume(value);
            }
        }
    }

    pub async fn callback_after_audio_ends(&self, callback: fn()) {
        if let Some(guard) = &self.sink {
            let temp = guard.clone();
            let (sender, receiver) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                if let Ok(sink) = temp.lock() {
                    sender
                        .send("(Audio) Executing after audio ends callback")
                        .unwrap();
                }
            });

            if let Ok(message) = receiver.recv() {
                println!("(Message) received");
                println!("{}", message);
                callback();
                // self.callback_after_audio_ends(callback);
            }
        }
    }
}

// fn open_stream() -> (OutputStream, Sink) {
    // let stream_handle =
    //     rodio::OutputStreamBuilder::open_default_stream().expect("Open default audio stream");
    // let sink = Sink::connect_new(stream_handle.mixer());
    // (stream_handle, sink)
// }

fn start_playing_from_stream() {}

fn start_playing_local_audio(sink: &Sink, audio_path: &Path) {
    // if let Ok(file) = File::open(audio_path) {
    //     match Decoder::try_from(file) {
    //         Ok(source) => {
    //             sink.append(source);
    //             sink.play();
    //         }
    //         Err(error) => {
    //             println!("(Audio) - start_playing_audio DecoderError: {:?}", error);
    //         }
    //     }
    // } else {
    //     println!("Couldn't open audio file: {:?}", audio_path);
    // }
}
