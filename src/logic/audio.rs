use rodio::{self, Decoder, OutputStream, Sink};
use std::{fs::File, path::Path, time::Duration};
use crate::State;

use super::database::{MediaFile, QueueItem};

// Rodio docs: https://docs.rs/rodio/latest/rodio/
// Note: The sound plays in a separate audio thread,
// so we need to keep the main thread alive while it's playing.

pub struct MediaPlayer {
	sink: Option<Sink>,
	output_stream: Option<OutputStream>
}

impl MediaPlayer {
	pub fn new() -> Self {
		Self { sink: None, output_stream: None }
	}

	fn new_sink(&mut self) {
		if let Some(output) = &self.output_stream {
			self.sink = Some(Sink::connect_new(output.mixer()));
		}
	}

	pub fn media_start(&mut self, audio: &MediaFile) {
		if self.sink.is_none() {
			let (output_stream, new_sink) = open_stream();
			self.sink = Some(new_sink);
			self.output_stream = Some(output_stream);
		}

		if let Some(sink) = &self.sink {
			let audio_path = Path::new(&audio.path);
			sink.set_volume(0.1);
			start_playing_audio(sink, audio_path);
		}
	}

	pub fn media_toggle(&mut self) {
		if self.sink.is_none() {
			return
		}

		if let Some(sink) = &self.sink {
			if sink.is_paused() {
				sink.play();
			} else {
				sink.pause();
			}
		}
	}

	pub fn media_pause(&mut self) {
		if let Some(sink) = &mut self.sink {
			pause_audio(sink);
		}
	}

	pub fn next_media(&self, state: &mut State) {
		if let Some(sink) = &self.sink {
			sink.skip_one();
			update_currently_playing(state, 1);
		}
	}

	pub fn previous_media(&mut self, state: &mut State) {
		if let Some(sink) = &self.sink {
			let current_audio: Vec<QueueItem> = state.queue
				.clone()
				.into_iter()
				.filter(|item| item.currently_playing == false)
				.collect();

			// let mut previous_audio = None;
			
			let mut index: i32 = 0;
			for audio in state.index.iter() {
				if audio.id == current_audio[0].media_id {
					break;
				}

				index += 1;
			}

			// if let Some(audio) = previous_audio {
			// 	// Destroy current queue
			// 	self.destroy_sink();
			// 	self.new_sink();
			// 	self.add_to_queue(state, audio.clone()); // Add the previous track to queue.
			// 	sink.play();

			// 	self.load_queue_from_state(state);
			// }

			// sink.append(source);
			update_currently_playing(state, -1);
		}
	}

	pub fn add_to_queue(&self, state: &mut State, media_file: &MediaFile) -> Result<(), ()> {
		if let Ok(file) = File::open(&media_file.path) {
			if let Ok(source) = Decoder::try_from(file) {
				if let Some(sink) = &self.sink {
					sink.append(source);
					state.queue.push(QueueItem {
						media_id: media_file.id,
						currently_playing: false,
					});

					return Ok(());
				}
			} else {
				println!("Can't add media to queue. Couldn't convert file.");
			}
		} else {
			println!("Can't add media to queue. Path might be unvalid: {}", media_file.path);
		}
	
		Err(())
	}

	fn load_queue_from_state(&self, state: &State) {
		for item in state.queue.iter() {
			
		}
	}

	pub fn current_track_position(sink: &Sink) -> u128 {
		sink.get_pos().as_millis()
	}
	
	pub fn clear_queue(sink: &Sink) {
		sink.clear();
	}
	
	pub fn set_volume(sink: &Sink, value: f32) {
		sink.set_volume(value);
	}
	
	pub fn current_track_position_change(sink: &Sink, position: Duration) {
		sink.try_seek(position);
	}
	
	pub fn destroy_sink(self) {
		if let Some(sink) = self.sink {
			sink.detach();
		}
	}
}

fn update_currently_playing(state: &mut State, update_direction: i32) {
	let mut index = 0;
	let next = update_direction > 0;
	let previous = update_direction < 0;
	let queue_length = state.queue.len();
	
	for item in state.queue.iter() {
		if item.currently_playing {
			if next {
				if index == queue_length {
					state.queue[0].currently_playing = true;
				} else {
					state.queue[index + 1].currently_playing = true;
				}
			} else if previous {
				if index == 0 {
					state.queue[queue_length - 1].currently_playing = true
				} else {
					state.queue[index - 1].currently_playing = true;
				}
			}
			
			state.queue[index].currently_playing = false;
			break
		}

		index += 1;
	}
}

fn open_stream() -> (OutputStream, Sink) {
	let output_stream = rodio::OutputStreamBuilder::open_default_stream()
		.expect("Open default audio stream");
	let sink = Sink::connect_new(output_stream.mixer());
	(output_stream, sink)
}

fn start_playing_audio(sink: &Sink, audio_path: &Path) {
	if let Ok(file) = File::open(audio_path) {
		let source = Decoder::try_from(file).unwrap();
		sink.append(source);
		sink.play();
	} else {
		println!("Couldn't open audio file: {:?}", audio_path);
	}
}

fn continue_audio(sink: &Sink) {
	if sink.is_paused() {
		sink.play();
	}
}

fn pause_audio(sink: &Sink) {
	if !sink.is_paused() {
		sink.pause();
	}
}
