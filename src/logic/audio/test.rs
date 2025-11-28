use crate::{logic::database::MediaFile, State};
use gstreamer::prelude::*;
use gstreamer::ElementFactory;

#[derive(Clone)]
pub struct MediaPlayer {
    pipeline: gstreamer::Pipeline,
    source: Option<gstreamer::Element>,
    decode_bin: gstreamer::Element,
    audio_convert: gstreamer::Element,
    audio_resample: gstreamer::Element,
    // auto_audio_sink: gstreamer::Element,
}

impl MediaPlayer {
    pub fn new() -> Self {
        gstreamer::init().unwrap();
        Self {
            pipeline: gstreamer::Pipeline::default(),
            source: None,
            decode_bin: ElementFactory::make("decodebin").build().unwrap(),
            audio_convert: ElementFactory::make("audioconvert").build().unwrap(),
            audio_resample: ElementFactory::make("audioresample").build().unwrap(),
            // auto_audio_sink: ElementFactory::make("autoaudiosink").build().unwrap(),
        }
    }

    pub fn change_source(
        &mut self,
        source_path: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.source = Some(
            ElementFactory::make("filesrc")
                .property("location", source_path)
                .build()?,
        );
        Ok(self.clone())
    }

    fn initialize_pipeline(&self) {
        self.pipeline
            .add_many(&[
                self.source.as_ref().unwrap(),
                &self.decode_bin,
                &self.audio_convert,
                &self.audio_resample,
                // &self.auto_audio_sink,
            ])
            .unwrap();
    }

    fn initialize_element_links(&self) {
        gstreamer::Element::link_many(&[&self.source.as_ref().unwrap(), &self.decode_bin]).unwrap();
        gstreamer::Element::link_many(&[
            &self.audio_convert,
            &self.audio_resample,
            // &self.auto_audio_sink,
        ])
        .unwrap();
    }

    fn clean_up(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pipeline.set_state(gstreamer::State::Null)?;
        Ok(())
    }

    fn watch(&self) {
        // Watch for an error or EOS
        let bus = self.pipeline.bus().unwrap();
        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            match msg.view() {
                gstreamer::MessageView::Error(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
                gstreamer::MessageView::Eos(..) => {
                    println!("End of stream");
                    break;
                }
                _ => (),
            }
        }
    }

    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize_pipeline();
        self.initialize_element_links();

        let audio_convert = self.audio_convert.clone();
        self.decode_bin
            .connect_pad_added(move |_decode_bin, source_pad| {
                let sink_pad = audio_convert.static_pad("sink").unwrap();
                if source_pad.link(&sink_pad).is_err() {
                    eprintln!("Failed to link pads"); // Should this panic, instead of giving an error?
                }
            });

        Ok(())
    }

    pub fn start(&mut self, audio: &MediaFile) {
        if self.source.is_none() {
            self.change_source(audio.path.clone()).unwrap();
        }
        self.pipeline.set_state(gstreamer::State::Playing).unwrap();
        self.watch(); // BUG that crashes the whole application without playing audio. Check pipeline.
        self.clean_up().unwrap();
    }

    pub fn start_next(&mut self, audio: &MediaFile) {}

    pub fn source_toggle(&mut self) {}

    pub fn pause(&mut self) {}

    pub fn source_change_to_specific(&mut self, index: i32) {}

    pub fn next(&self) {}

    pub fn previous(&mut self, state: &mut State) {}

    pub fn clear_queue(&self) {}

    // pub fn create_queue(&self, media_files: Vec<&MediaFile>) -> Result<(), ()> {}

    // pub fn add_to_queue(&self, state: &mut State, media_file: &MediaFile) -> Result<(), ()> {}

    fn load_queue_from_state(&mut self, state: &State) {}

    // pub fn change_current_track_position(&mut self, position: Duration) {}

    pub fn get_current_track_position(&self) -> u32 {
        // if let Some(guard) = &self.sink {
        //     if let Ok(sink) = guard.lock() {
        //         return sink.get_pos().as_secs_f32() as u32;
        //     }
        // }

        0
    }

    pub fn set_volume(&self, value: f32) {}

    pub async fn callback_after_audio_ends(&self, callback: fn()) {}
}
