use crate::State;
use super::database::{MediaFile, QueueItem};

pub trait Queue {
	fn add_to_queue(&mut self, media: &MediaFile);
	fn remove_from_queue(&mut self, id: i32);
	fn progress_queue(&mut self) -> Vec<QueueItem>;
	fn move_to_specific_index_from_current(&mut self, index: i32) -> Option<(usize, usize)>;
}

impl Queue for State {
	fn add_to_queue(&mut self, media: &MediaFile) {
		let mut item = QueueItem {
			media_id: media.id,
			currently_playing: false,
		};

		if self.queue.is_empty() {
			item.currently_playing = true;
		}

		self.queue.push(item);
	}

	fn remove_from_queue(&mut self, id: i32) {
		let found = self.queue.iter().position(|item| item.media_id == id);

		if let Some(index) = found {
			self.queue.remove(index);
		}
	}

	fn progress_queue(&mut self) -> Vec<QueueItem> {
		let found = self.queue.iter().position(|item| item.currently_playing);

		if let Some(index) = found {
			self.queue[index].currently_playing = false;
			let queue_length = self.queue.len();

			if index + 1 < queue_length {
				self.queue[index + 1].currently_playing = true;
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
			.iter_mut()
			.enumerate()
			.find(|(_, media)| media.currently_playing) {
				if (index as i32 + move_index) < self.queue.len() as i32 {
					let sum_index = (index as i32 + move_index) as usize;
					self.queue[index].currently_playing = false;
					self.queue[sum_index].currently_playing = true;
					return Some((index, sum_index));
				}
			}
		
		None
	}
}