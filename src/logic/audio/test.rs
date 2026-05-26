use std::{sync::{Arc, Mutex}, thread};
use crate::{logic::data_types::track::Track, State as AppState};
use derive_more::Display;
use symphonia::core::{
	codecs::Decoder,
	formats::FormatOptions,
	io::MediaSourceStream,
	meta::MetadataOptions,
	probe::Hint
};
use symphonia_core::{
	codecs::audio::AudioDecoderOptions,
	errors::{Error, Result},
	formats::TrackType,
	audio
};
use thiserror::Error;

#[derive(Debug, Display, Error)]
#[display("Received error from {src}: {error} (debug: {debug:?})")]
struct ErrorMessage {
    src: glib::GString,
    error: glib::Error,
    debug: Option<glib::GString>,
}

const CHUNK_SIZE: usize = 1024;

#[derive(Clone)]
pub struct Player {
	hint: Result<Hint>,
	stream: Result<MediaSourceStream>,
	source: Result<std::fs::File>,
	decoder: Result<Box<dyn Decoder>>,
}

impl Player {
    pub fn new() -> Self {
        Self {
        	stream: None,
         	source: None,
          	hint: None,
            decoder: None,
        }
    }

    pub fn initialize(&mut self, source_path: String) {
     	// TODO: Check if local or not + add better error handler.
      	let (source, _, _, _) = self.set_new_source(source_path);
    	let stream = MediaSourceStream::new(Box::new(source), Default::default());
    	if let Err(err) = !self.set_decoder(self.source) {
   			println!("Player :: Initialization error: {}", err)
     	}
    }

    fn set_new_source(&mut self, source_path: String) -> Result<(std::fs::File, Hint, MetadataOptions, FormatOptions)> {
    	let source = std::fs::File::open(source_path);
     	if let Err(err) = &source {
    		println!("Player :: Error: failed to open media");
      		return err;
      	}

    	let mut hint = Hint::new();
     	hint.with_extension("ogg"); // TODO: Probe from custom source file struct
      	let meta_opts: MetadataOptions = Default::default();
       	let fmt_opts: FormatOptions = Default::default();

        self.source = source;
        Ok((source, hint, meta_opts, fmt_opts))
    }

    fn set_decoder(&mut self, source: Result<std::fs::File>) -> Result<()> {
	    let mut format = symphonia::default::get_probe()
	        .probe(&self.hint)
	       	.expect("Player :: unsupported format");
	    let track = format.default_track(TrackType::Audio);
		if let Err(err) = &track {
			println!("Player :: Error: {}", err);
			return err;
		}

	    let mut dec_opts: AudioDecoderOptions = Default::default();
	    let mut decoder = symphonia::default::get_codecs().make(
	  		track.codec_params.as_ref().expect("Player :: codec parameters missing").audio().unwrap(),
	       	&dec_opts,
	   	).expect("Player :: decoder: unsupported codec.");

		self.decoder = decoder;
		Ok(())
    }

    pub fn play(&mut self, new_source: Option<String>) {
    	if new_source != self.source {
     		self.set_new_source(new_source);
     	}

    	thread::spawn(|| {
	    	loop {
	     		let packet = match self.format.next_packet() {
					Ok(Some(packet)) => packet,
					Ok(None) => {
						// Reached the end of the stream.
						break;
					}
					Err(Error::ResetRequired) => {
						// The track list has been changed. Re-examine it and create a new set of decoders,
						// then restart the decode loop. This is an advanced feature and it is not
						// unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
						// for chained OGG physical streams.
						unimplemented!();
					}
					Err(err) => {
						// A unrecoverable error occurred, halt decoding.
						panic!("{}", err);
					}
	       		};

         		// Consumes latest metadata and pop the old head
	       		// while !self.format.metadata().is_latest() {
	         	// 	self.format.metadata().pop();
				// }

	       		if packet.track_id != self.source.id {
	         		continue;
	         	}

	     		match self.decoder.decode(&packet) {
	    			Ok(_decoded) => {
		       			// Consume the decoded sample
		       		}
		         	Err(Error::IoError(_)) => {
		           	// The packet failed to decode due to an IO error, skip the packet.
		                continue;
		            }
		            Err(Error::DecodeError(_)) => {
		                // The packet failed to decode due to invalid data, skip the packet.
		                continue;
		            }
					Err(err) => {
						panic!("Player (panic) ::: {}", err);
					}
	       		}
	     	}
     	});
    }

    pub fn next(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn previous(&mut self, state: &mut AppState) {}

    pub fn source_change_to_specific(&mut self, index: i32) {}

    pub fn clear_queue(&self) {}

    // pub fn create_queue(&self, media_files: Vec<&Track>) -> Result<(), ()> {}

    // pub fn add_to_queue(&self, state: &mut State, media_file: &Track) -> Result<(), ()> {}

    // fn load_queue_from_state(&mut self, state: &AppState) {}

    // pub fn change_current_track_position(&mut self, position: Duration) {}

    pub fn get_current_track_position(&self) -> u32 {
        0
    }

    pub fn set_volume(&self, value: f64) {
        self.volume.set_property("volume", value);
    }

    pub async fn callback_after_audio_ends(&self, callback: fn()) {}
}
