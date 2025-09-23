// Custom sink that offers more flexible way to detect what is happening in the media player.
// In this approach queue is handled by sink.

use std::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(feature = "crossbeam-channel")]
use crossbeam_channel::{Receiver, Sender};
use dasp_sample::FromSample;
#[cfg(not(feature = "crossbeam-channel"))]
use std::sync::mpsc::{Receiver, Sender};

use rodio::mixer::Mixer;
// use dasp_sample::FromSample;
use rodio::{queue, Source};
use rodio::source::{Done, SeekError};

pub struct Sink {
	queue_tx: Arc<queue::SourcesQueueInput>,
	sleep_until_end: Mutex<Option<Receiver<()>>>,

	controls: Arc<Controls>,
	sound_count: Arc<AtomicUsize>,

	detached: bool,
}

struct SeekOrder {
    pos: Duration,
    feedback: Sender<Result<(), SeekError>>,
}

impl SeekOrder {
    fn new(pos: Duration) -> (Self, Receiver<Result<(), SeekError>>) {
        #[cfg(not(feature = "crossbeam-channel"))]
        let (tx, rx) = {
            use std::sync::mpsc;
            mpsc::channel()
        };

        #[cfg(feature = "crossbeam-channel")]
        let (tx, rx) = {
            use crossbeam_channel::bounded;
            bounded(1)
        };
        (Self { pos, feedback: tx }, rx)
    }

    fn attempt<S>(self, maybe_seekable: &mut S)
    where
        S: Source,
    {
        let res = maybe_seekable.try_seek(self.pos);
        let _ignore_receiver_dropped = self.feedback.send(res);
    }
}

struct Controls {
    pause: AtomicBool,
    volume: Mutex<f32>,
    stopped: AtomicBool,
    speed: Mutex<f32>,
    to_clear: Mutex<u32>,
    seek: Mutex<Option<SeekOrder>>,
    position: Mutex<Duration>,
}

impl Sink {
	pub fn connect_new(mixer: &Mixer) -> Sink {
        let (sink, source) = Sink::new();
        mixer.add(source);
        sink
    }

    pub fn new() -> (Sink, queue::SourcesQueueOutput) {
        let (queue_tx, queue_rx) = queue::queue(true);

        let sink = Sink {
            queue_tx,
            sleep_until_end: Mutex::new(None),
            controls: Arc::new(Controls {
                pause: AtomicBool::new(false),
                volume: Mutex::new(1.0),
                stopped: AtomicBool::new(false),
                speed: Mutex::new(1.0),
                to_clear: Mutex::new(0),
                seek: Mutex::new(None),
                position: Mutex::new(Duration::ZERO),
            }),
            sound_count: Arc::new(AtomicUsize::new(0)),
            detached: false,
        };
        (sink, queue_rx)
    }

	pub fn append<S: Source + Send + 'static>(&self, source: S) {
		if self.controls.stopped.load(Ordering::SeqCst) {
            if self.sound_count.load(Ordering::SeqCst) > 0 {
                self.sleep_until_end();
            }
            self.controls.stopped.store(false, Ordering::SeqCst);
        }

		let controls = self.controls.clone();
        let start_played = AtomicBool::new(false);

		let source = source
			.speed(1.0)
			// Must be placed before pausable but after speed & delay
			.track_position()
			.pausable(false)
			.amplify(1.0)
			.skippable()
			.stoppable()
			// If you change the duration update the docs for try_seek!
			.periodic_access(Duration::from_millis(5), move |src| {
				if controls.stopped.load(Ordering::SeqCst) {
					src.stop();
					*controls.position.lock().unwrap() = Duration::ZERO;
				}

				{
					let mut to_clear = controls.to_clear.lock().unwrap();
					if *to_clear > 0 {
						src.inner_mut().skip();
						*to_clear -= 1;
						*controls.position.lock().unwrap() = Duration::ZERO;
					} else {
						*controls.position.lock().unwrap() = src.inner().inner().inner().inner().get_pos();
					}
				}

				let amp = src.inner_mut().inner_mut();
				amp.set_factor(*controls.volume.lock().unwrap());
				amp.inner_mut()
					.set_paused(controls.pause.load(Ordering::SeqCst));
				amp.inner_mut()
					.inner_mut()
					.inner_mut()
					.set_factor(*controls.speed.lock().unwrap());
				if let Some(seek) = controls.seek.lock().unwrap().take() {
					seek.attempt(amp)
				}
				start_played.store(true, Ordering::SeqCst);
			});

		self.sound_count.fetch_add(1, Ordering::Relaxed);
		let source = Done::new(source, self.sound_count.clone());
		*self.sleep_until_end.lock().unwrap() = Some(self.queue_tx.append_with_signal(source));
	}

	pub fn volume(&self) -> f32 {
        *self.controls.volume.lock().unwrap()
    }

	pub fn set_volume(&self, value: f32) {
        *self.controls.volume.lock().unwrap() = value;
    }

	pub fn speed(&self) -> f32 {
        *self.controls.speed.lock().unwrap()
    }

	pub fn set_speed(&self, value: f32) {
        *self.controls.speed.lock().unwrap() = value;
    }

	pub fn play(&self) {
        self.controls.pause.store(false, Ordering::SeqCst);
    }

	pub fn try_seek(&self, pos: Duration) -> Result<(), SeekError> {
        let (order, feedback) = SeekOrder::new(pos);
        *self.controls.seek.lock().unwrap() = Some(order);

        if self.sound_count.load(Ordering::Acquire) == 0 {
            // No sound is playing, seek will not be performed
            return Ok(());
        }

        match feedback.recv() {
            Ok(seek_res) => {
                *self.controls.position.lock().unwrap() = pos;
                seek_res
            }
            // The feedback channel closed. Probably another SeekOrder was set
            // invalidating this one and closing the feedback channel
            // ... or the audio thread panicked.
            Err(_) => Ok(()),
        }
    }

	pub fn pause(&self) {
        self.controls.pause.store(true, Ordering::SeqCst);
    }

	pub fn is_paused(&self) -> bool {
        self.controls.pause.load(Ordering::SeqCst)
    }

	pub fn clear(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        *self.controls.to_clear.lock().unwrap() = len;
        // self.sleep_until_end();
        // self.pause();
    }

	pub fn skip_one(&self) {
        let len = self.sound_count.load(Ordering::SeqCst) as u32;
        let mut to_clear = self.controls.to_clear.lock().unwrap();
        if len > *to_clear {
            *to_clear += 1;
        }
    }

	pub fn stop(&self) {
        self.controls.stopped.store(true, Ordering::SeqCst);
    }

	pub fn detach(mut self) {
        self.detached = true;
    }

	pub fn sleep_until_end(&self) {
        if let Some(sleep_until_end) = self.sleep_until_end.lock().unwrap().take() {
            let _ = sleep_until_end.recv();
        }
    }

	pub fn empty(&self) -> bool {
        self.len() == 0
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.sound_count.load(Ordering::Relaxed)
    }

	pub fn get_pos(&self) -> Duration {
        *self.controls.position.lock().unwrap()
    }
}

impl Drop for Sink {
    #[inline]
    fn drop(&mut self) {
        self.queue_tx.set_keep_alive_if_empty(false);

        if !self.detached {
            self.controls.stopped.store(true, Ordering::Relaxed);
        }
    }
}
