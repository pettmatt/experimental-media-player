use std::collections::HashMap;
use rand::prelude::*;
use crate::State;
use super::database::{MediaFile, QueueItem};

pub trait Queue {
	fn add_to_queue(&mut self, media: &MediaFile);
	fn remove_from_queue(&mut self, id: i32);
	fn progress_queue(&mut self) -> Vec<QueueItem>;
	fn move_to_specific_index_from_current(&mut self, index: i32) -> Option<(usize, usize)>;
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
				return track.currently_playing;
			}

			false
		});

		if let Some(index) = found {
			let queue_length = self.queue.len();

			if index + 1 < queue_length {
				{
					let index_position = self.index.iter().position(|item| item.id == self.queue[index].media_id).unwrap();
					if let Some(track) = self.index.get_mut(index_position) {
						track.currently_playing = false;
					}
				}

				let index_position = self.index.iter().position(|item| item.id == self.queue[index + 1].media_id).unwrap();

				if let Some(track) = self.index.get_mut(index_position) {
					track.currently_playing = true;
				}
			}

			// Either loop, add x amount of random tracks to queue from index (if the queue has ended) 
			// or just stop playing audio.
			// Currently, does nothing.

			return self.queue[index..queue_length].to_vec();
		}

		Vec::new()
	}

	fn move_to_specific_index_from_current(&mut self, move_index: i32) -> Option<(usize, usize)> {
		if let Some((index, _)) = self.queue
			.iter()
			.enumerate()
			.find(|(_, item)| self.index[item.media_id as usize].currently_playing) {
				if (index as i32 + move_index) < self.queue.len() as i32 {
					let sum_index = (index as i32 + move_index) as usize;
					let media_index = self.queue[index].media_id as usize;
					self.index[media_index].currently_playing = false;

					let media_index = self.queue[sum_index].media_id as usize;
					self.index[media_index].currently_playing = true;
					return Some((index, sum_index));
				}
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