use std::collections::HashMap;
use rand::prelude::*;
use crate::State;
use super::database::{MediaFile, QueueItem};

pub trait Queue {
	fn add_to_queue(&mut self, media: &MediaFile);
	fn remove_from_queue(&mut self, id: i32);
	fn progress_queue(&mut self) -> Vec<QueueItem>;
	fn update_playing_audio_in_queue(&mut self, index: i32) -> Option<(usize, usize)>;
	fn shuffle(&mut self);
}

impl Queue for State {
	fn add_to_queue(&mut self, media: &MediaFile) {
		self.queue.push(QueueItem {
			media_id: media.id,
		});
	}

	fn remove_from_queue(&mut self, id: i32) {
		let found = self.queue.iter().position(|item| item.media_id == id);

		if let Some(index) = found {
			self.queue.remove(index);
		}
	}

	fn progress_queue(&mut self) -> Vec<QueueItem> {
		let index_map: HashMap<i32, &MediaFile> = self.index.iter().map(|item| (item.id, item)).collect();
		let found: Option<usize> = self.queue.iter().position(|item| {
			if let Some(track) = index_map.get(&item.media_id) {
				return track.playing;
			}

			false
		});

		if let Some(index) = found {
			let queue_length = self.queue.len();

			if index + 1 < queue_length {
				{
					let index_position = self.index.iter().position(|item| item.id == self.queue[index].media_id).unwrap();
					if let Some(track) = self.index.get_mut(index_position) {
						track.playing = false;
					}
				}

				let index_position = self.index.iter().position(|item| item.id == self.queue[index + 1].media_id).unwrap();

				if let Some(track) = self.index.get_mut(index_position) {
					track.playing = true;
				}
			}

			// Either loop, add x amount of random tracks to queue from index (if the queue has ended) 
			// or just stop playing audio.
			// Currently, does nothing.

			return self.queue[index..queue_length].to_vec();
		}

		Vec::new()
	}

	fn update_playing_audio_in_queue(&mut self, offset: i32) -> Option<(usize, usize)> {
		let index = self.playing.queue_index.unwrap();
		let next_index = index + offset as usize;
		// There might be cases where audio is not playing before this function is executed.
		self.set_index_playing(index, false);

		if self.set_index_playing(next_index, true).is_some() {
			return Some((index, next_index));
		}

		None
	}

	fn shuffle(&mut self) {
		let mut random = rand::rng();
		let length = self.queue.len();

		for i in (1..length).rev() {
			let j = random.random_range(0..=i);
			self.queue.swap(i, j);
		}
	}
}
