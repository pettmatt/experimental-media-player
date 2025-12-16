use crate::{logic::data_types::track::Track, State as AppState};
use anyhow::Error;
use derive_more::derive::{Display, Error};
use gstreamer::{element_error, element_warning, prelude::*, State};
use gstreamer::{Element, ElementFactory};
use std::sync::{Arc, Mutex};

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
    pipeline: gstreamer::Pipeline, // DO NOT USE pipeline.clone() TO USE THE PIPELINE WITHIN A CALLBACK
    source: Option<Element>,
    decode_bin: Element,
    volume: Element,
}

impl MediaPlayer {
    pub fn new() -> Self {
        gstreamer::init().unwrap();
        Self {
            pipeline: gstreamer::Pipeline::default(),
            source: None,
            decode_bin: ElementFactory::make("decodebin").build().unwrap(),
            volume: ElementFactory::make("volume").build().unwrap(),
        }
    }

    pub fn change_source(&mut self, source_path: String) -> Result<(), Box<dyn std::error::Error>> {
        self.source = Some(
            ElementFactory::make("filesrc")
                .property("location", source_path)
                .build()?,
        );
        Ok(())
    }

    fn initialize_pipeline(&self) {
        self.pipeline
            .add_many(&[self.source.as_ref().unwrap(), &self.decode_bin])
            .unwrap();
    }

    fn initialize_element_links(&self) {
        Element::link_many([&self.source.as_ref().unwrap(), &self.decode_bin]).unwrap();
    }

    fn clean_up(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pipeline.set_state(State::Null)?;
        Ok(())
    }

    fn watch(&self, bus: gstreamer::Bus) {
        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            use gstreamer::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                // MessageView::Error(err) => {
                //     self.pipeline.set_state(State::Null).unwrap();

                //     match err.details() {
                //         Some(details) if details.name() == "error-details" => details
                //             .get::<&ErrorValue>("error")
                //             .unwrap()
                //             .clone()
                //             .0
                //             .lock()
                //             .unwrap()
                //             .take()
                //             .map(anyhow::Result::Err)
                //             .expect("error-details message without actual error"),
                //         _ => Err(ErrorMessage {
                //             src: msg
                //                 .src()
                //                 .map(|s| s.path_string())
                //                 .unwrap_or_else(|| glib::GString::from("UNKNOWN")),
                //             error: err.error(),
                //             debug: err.debug(),
                //         }
                //         .into()),
                //     }
                //     .unwrap();
                // }
                MessageView::StateChanged(s) => {
                    println!(
                        "State changed from {:?}: {:?} -> {:?} ({:?})",
                        s.src().map(|s| s.path_string()),
                        s.old(),
                        s.current(),
                        s.pending()
                    );
                }
                _ => (),
            }
        }

        self.pipeline.set_state(State::Null).unwrap();

        // Watch for an error or EOS
        // let bus = self.pipeline.bus().unwrap();
        // for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
        //     match msg.view() {
        //         gstreamer::MessageView::Error(err) => {
        //             eprintln!("Error: {:?}", err);
        //             break;
        //         }
        //         gstreamer::MessageView::Eos(..) => {
        //             println!("End of stream");
        //             break;
        //         }
        //         _ => (),
        //     }
        // }
    }

    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize_pipeline();
        self.initialize_element_links();

        let pipeline_weak = self.pipeline.downgrade();
        self.decode_bin
            .connect_pad_added(move |decode_bin, source_pad| {
                let Some(pipeline) = pipeline_weak.upgrade() else {
                    return;
                };

                let (is_audio, is_video) = {
                    let media_type = source_pad.current_caps().and_then(|caps| {
                        caps.structure(0).map(|string| {
                            let name = string.name();
                            (name.starts_with("audio/"), name.starts_with("video/"))
                        })
                    });

                    match media_type {
                        Some(media_type) => media_type,
                        None => {
                            element_warning!(
                                decode_bin,
                                gstreamer::CoreError::Negotiation,
                                ("Failed to get media type from pad {}", source_pad.name())
                            );
                            return;
                        }
                    }
                };

                let insert_sink = |is_audio, is_video| -> Result<(), Error> {
                    if is_audio {
                        let queue = ElementFactory::make("queue").build().unwrap();
                        let convert = ElementFactory::make("audioconvert").build().unwrap();
                        let resample = ElementFactory::make("audioresample").build().unwrap();
                        let sink = ElementFactory::make("autoaudiosink").build().unwrap(); // pwaudiosink | autoaudiosink

                        let elements = &[&queue, &convert, &resample, &sink];
                        pipeline.add_many(elements).unwrap();
                        Element::link_many(elements).unwrap();
                    } else if is_video {
                        let queue = ElementFactory::make("queue").build().unwrap();
                        let convert = ElementFactory::make("videoconvert").build().unwrap();
                        let scale = ElementFactory::make("videoscale").build().unwrap();
                        let sink = ElementFactory::make("autovideosink").build().unwrap();

                        let elements = &[&queue, &convert, &scale, &sink];
                        pipeline.add_many(elements).unwrap();
                        Element::link_many(elements).unwrap();

                        for element in elements {
                            element.sync_state_with_parent().unwrap()
                        }

                        let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
                        source_pad.link(&sink_pad).unwrap();
                    }

                    Ok(())
                };

                if let Err(err) = insert_sink(is_audio, is_video) {
                    element_error!(
                        decode_bin,
                        gstreamer::LibraryError::Failed,
                        ("Failed to insert sink"),
                        details: gstreamer::Structure::builder("error-details")
                            .field("error", ErrorValue(Arc::new(Mutex::new(Some(err)))))
                            .build()
                    );
                }
            });

        Ok(())
    }

    pub fn start(&mut self, audio: &Track) {
        if self.source.is_none() {
            self.change_source(audio.path.clone()).unwrap();
        }

        if let Ok(()) = self.initialize() {
            println!("(MediaPlayer) Initialization DONE")
        }

        self.set_volume(0.5f64);
        self.pipeline.set_state(State::Playing).unwrap();
        let bus = self
            .pipeline
            .bus()
            .expect("Oops... pipeline should always have access to bus!");
        self.watch(bus);

        // if self.source.is_none() {
        //     self.change_source(audio.path.clone()).unwrap();
        // }
        // self.pipeline.set_state(gstreamer::State::Playing).unwrap();
        // self.watch(); // BUG that crashes the whole application without playing audio. Check pipeline.
        // self.clean_up().unwrap();
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
