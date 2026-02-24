use crate::{logic::data_types::track::Track, State as AppState};
use anyhow::Error;
use derive_more::derive::{Display, Error};
// use gstreamer::{element_error, element_warning, prelude::*, State};
// use gstreamer::{Element, ElementFactory};
use std::sync::{Arc, Mutex};
use std::env;


// Source: https://gitlab.freedesktop.org/gstreamer/gstreamer-rs/-/blob/main/examples/src/bin/decodebin.rs?ref_type=heads

#[derive(Debug, Display, Error)]
#[display("Received error from {src}: {error} (debug: {debug:?})")]
struct ErrorMessage {
    src: glib::GString,
    error: glib::Error,
    debug: Option<glib::GString>,
}

#[derive(Clone, Debug, glib::Boxed)]
#[boxed_type(name = "ErrorValue")]
struct ErrorValue(Arc<Mutex<Option<Error>>>);

#[derive(Clone)]
pub struct MediaPlayer {
    // pipeline: std::sync::Arc<gstreamer::Pipeline>, // DO NOT USE pipeline.clone() TO USE THE PIPELINE WITHIN A CALLBACK
    // source: Option<Element>,
    // decode_bin: Element,
    // volume: Element,
    thread: Option<std::thread::Thread>
}

impl MediaPlayer {
    pub fn new() -> Self {
        // gstreamer::init().unwrap();
        Self {
            // pipeline: std::sync::Arc::new(gstreamer::Pipeline::default()),
            // source: None,
            // decode_bin: ElementFactory::make("decodebin").build().unwrap(),
            // volume: ElementFactory::make("volume").build().unwrap(),
            thread: None,
        }
    }

    pub fn change_source(&mut self, source_path: String) -> Result<(), Box<dyn std::error::Error>> {
    	if let Some(source) = &self.source {
     		source.set_property("location", &source_path);
     	} else {
	        // self.source = Some(
	            // ElementFactory::make("filesrc")
	            //     .property("location", source_path)
	            //     .build()?,
	        // );
      	}
        Ok(())
    }

    // fn initialize_pipeline(&self) {
    //     self.pipeline
    //         .add_many(&[self.source.as_ref().unwrap(), &self.decode_bin])
    //         .unwrap();
    // }

    // fn initialize_element_links(&self) {
    //     Element::link_many([&self.source.as_ref().unwrap(), &self.decode_bin]).unwrap();
    // }

    // fn clean_up(&self) -> Result<(), Box<dyn std::error::Error>> {
    //     self.pipeline.set_state(State::Null)?;
    //     Ok(())
    // }

    // fn watch(&self, bus: gstreamer::Bus) {

    // }

    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    	// gstreamer::init()?;

	    let args: Vec<_> = env::args().collect();
	    let uri: &str = if args.len() == 2 {
			args[1].as_ref()
	    } else {
	        println!("Usage: decodebin file_path");
	        std::process::exit(-1)
	     };

	    // let pipeline = gstreamer::Pipeline::default();
	    // let src = gstreamer::ElementFactory::make("filesrc")
	    //     .property("location", uri)
	    //     .build()?;
	    // let decodebin = gstreamer::ElementFactory::make("decodebin").build()?;

		self.change_source(uri.to_string());

	    self.pipeline.add_many(&[self.source.as_ref().unwrap(), &self.decode_bin])?;
	    // gstreamer::Element::link_many([self.source.as_ref().unwrap(), &self.decode_bin])?;

	    let pipeline_weak = self.pipeline.downgrade();
	    self.decode_bin.connect_pad_added(move |dbin, src_pad| {
	        let Some(pipeline) = pipeline_weak.upgrade() else {
	            return;
	        };

	        let (is_audio, is_video) = {
	            let media_type = src_pad.current_caps().and_then(|caps| {
	                caps.structure(0).map(|s| {
	                    let name = s.name();
	                    (name.starts_with("audio/"), name.starts_with("video/"))
	                })
	            });

	            match media_type {
	                None => {
	                    // element_warning!(
	                    //     dbin,
	                    //     gstreamer::CoreError::Negotiation,
	                    //     ("Failed to get media type from pad {}", src_pad.name())
	                    // );

	                    return;
	                }
	                Some(media_type) => media_type,
	            }
	        };

	        let insert_sink = |is_audio, is_video| -> Result<(), Error> {
	            // if is_audio {
	                // let queue = gstreamer::ElementFactory::make("queue").build()?;
	                // let convert = gstreamer::ElementFactory::make("audioconvert").build()?;
	                // let resample = gstreamer::ElementFactory::make("audioresample").build()?;
	                // let sink = gstreamer::ElementFactory::make("autoaudiosink").build()?;

	                // let elements = &[&queue, &convert, &resample, &sink];
	                // pipeline.add_many(elements)?;
	                // gstreamer::Element::link_many(elements)?;

	            //     for e in elements {
	            //         e.sync_state_with_parent()?;
	            //     }

	            //     let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
	            //     src_pad.link(&sink_pad)?;
	            // } else if is_video {
	            //     let queue = gstreamer::ElementFactory::make("queue").build()?;
	            //     let convert = gstreamer::ElementFactory::make("videoconvert").build()?;
	            //     let scale = gstreamer::ElementFactory::make("videoscale").build()?;
	            //     let sink = gstreamer::ElementFactory::make("autovideosink").build()?;

	            //     let elements = &[&queue, &convert, &scale, &sink];
	            //     pipeline.add_many(elements)?;
	            //     gstreamer::Element::link_many(elements)?;

	            //     for e in elements {
	            //         e.sync_state_with_parent()?
	            //     }

	            //     let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
	            //     src_pad.link(&sink_pad)?;
	            // }

	            Ok(())
	        };

	        if let Err(err) = insert_sink(is_audio, is_video) {
	            // element_error!(
	            //     dbin,
	            //     gstreamer::LibraryError::Failed,
	            //     ("Failed to insert sink"),
	            //     details: gstreamer::Structure::builder("error-details")
             //            .field("error", ErrorValue(Arc::new(Mutex::new(Some(err)))))
             //            .build()
	            // );
	        }
	    });

		let pipeline = std::sync::Arc::clone(&self.pipeline);
		std::thread::spawn(move || {
			// pipeline.set_state(gstreamer::State::Playing).unwrap();

			let bus = pipeline
			    .bus()
			    .expect("Pipeline without bus. Shouldn't happen!");

			// for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
			//     use gstreamer::MessageView;

			//     match msg.view() {
			//         MessageView::Eos(..) => break,
			//         MessageView::Error(err) => {
			//             pipeline.set_state(gstreamer::State::Null).unwrap();
			//             eprintln!("(MediaPlayer bus error): {:?}", err);
			//            	break;
			//         }
			//         MessageView::StateChanged(s) => {
			//             println!(
			//                 "(MediaPlayer) state changed from {:?}: {:?} -> {:?} ({:?})",
			//                 s.src().map(|s| s.path_string()),
			//                 s.old(),
			//                 s.current(),
			//                 s.pending()
			//             );
			//         }
			//         _ => (),
			//     }
			// }
			// pipeline.set_state(gstreamer::State::Null).unwrap();
		});

	    Ok(())
    }

    pub fn start(&mut self, audio: &Track) {
        if self.source.is_none() {
            self.change_source(audio.path.clone()).unwrap();
        }

        self.set_volume(0.5f64);

        if let Ok(()) = self.initialize() {
            println!("(MediaPlayer) Initialization DONE")
        }

        // self.pipeline.set_state(State::Playing).unwrap();
        // let bus = self
        //     .pipeline
        //     .bus()
        //     .expect("Oops... pipeline should always have access to bus!");
    }

    pub fn start_next(&mut self, audio: &Track) {}

    pub fn source_toggle(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn source_change_to_specific(&mut self, index: i32) {}

    pub fn next(&self) {}

    pub fn previous(&mut self, state: &mut AppState) {}

    pub fn clear_queue(&self) {}

    // pub fn create_queue(&self, media_files: Vec<&Track>) -> Result<(), ()> {}

    // pub fn add_to_queue(&self, state: &mut State, media_file: &Track) -> Result<(), ()> {}

    fn load_queue_from_state(&mut self, state: &AppState) {}

    // pub fn change_current_track_position(&mut self, position: Duration) {}

    pub fn get_current_track_position(&self) -> u32 {
        // if let Some(guard) = &self.sink {
        //     if let Ok(sink) = guard.lock() {
        //         return sink.get_pos().as_secs_f32() as u32;
        //     }
        // }

        0
    }

    pub fn set_volume(&self, value: f64) {
       	self.volume.set_property("volume", value);
    }

    pub async fn callback_after_audio_ends(&self, callback: fn()) {}
}
