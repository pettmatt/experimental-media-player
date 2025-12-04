use std::{
    env,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use derive_more::derive::{Display, Error};
use gstreamer::{Element, ElementFactory, State};
use gstreamer::{element_error, element_warning, prelude::*};

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

#[derive(Clone, Default, Debug)]
struct Track {
    url: String,
    played_timestamp: i32,
}

#[derive(Clone, Debug)]
pub struct MediaPlayer {
    pipeline: gstreamer::Pipeline,
    source: Element,
    decode_bin: Element,
    volume: Element,
    playedlist: Vec<String>,
    playlist: Vec<String>,
    playing: Track,
}

impl MediaPlayer {
    pub fn new() -> Self {
        Self {
            pipeline: gstreamer::Pipeline::default(),
            source: ElementFactory::make("filesrc").build().unwrap(),
            decode_bin: ElementFactory::make("decodebin").build().unwrap(),
            volume: ElementFactory::make("volume").build().unwrap(),
            playlist: Vec::new(),
            playedlist: Vec::new(),
            playing: Track::default(),
        }
    }

    fn clean_up(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pipeline.set_state(State::Null)?;
        Ok(())
    }

    fn watch(&self, bus: gstreamer::Bus) -> Result<(), Box<dyn std::error::Error>> {
        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            use gstreamer::MessageView;

            match msg.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(err) => {
                    self.pipeline.set_state(gstreamer::State::Null)?;

                    match err.details() {
                        Some(details) if details.name() == "error-details" => details
                            .get::<&ErrorValue>("error")
                            .unwrap()
                            .clone()
                            .0
                            .lock()
                            .unwrap()
                            .take()
                            .map(Result::Err)
                            .expect("error-details message without actual error"),
                        _ => Err(ErrorMessage {
                            src: msg
                                .src()
                                .map(|s| s.path_string())
                                .unwrap_or_else(|| glib::GString::from("UNKNOWN")),
                            error: err.error(),
                            debug: err.debug(),
                        }
                        .into()),
                    }?;
                }
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

        Ok(())
    }

    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.pipeline.add_many([&self.source, &self.decode_bin])?;
        Element::link_many([&self.source, &self.decode_bin])?;

        Ok(())
    }

    pub fn start(&mut self, track: String) {
        // Bug: If the source is not set at the start set_state() will create an error.
        self.pipeline.set_state(State::Paused).unwrap();
        if self
            .source
            .has_property_with_type("location", glib::Type::INVALID)
        {
            println!("Invalid");
            self.change_source(track.clone());
        } else {
            println!("Invalid else {:?}", self.source);
            self.set_source(track.clone());
            println!("Invalid else 2 {:?}", self.source);
            self.initialize().unwrap();
        }

        self.set_volume(0.1f64);
        self.pipeline
            .seek_simple(
                gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT,
                gstreamer::ClockTime::ZERO,
            )
            .unwrap();
        self.pipeline.set_state(State::Playing).unwrap();
        let bus = self
            .pipeline
            .bus()
            .expect("Oops... pipeline should always have access to bus!");

        if self.watch(bus).is_ok() {
            self.clean_up().unwrap();
        }
    }

    fn set_source(&mut self, source_path: String) {
        self.source = ElementFactory::make("filesrc")
            .property("location", source_path)
            .build()
            .unwrap();
    }

    pub fn change_source(&mut self, source_path: String) {
        self.source.set_property("location", source_path);
    }

    pub fn set_volume(&self, value: f64) {
        self.volume.set_property("volume", value);
    }
}

fn example_main() -> Result<(), Error> {
    gstreamer::init()?;

    let args: Vec<_> = env::args().collect();
    let uri: &str = if args.len() == 2 {
        args[1].as_ref()
    } else {
        println!("Usage: decodebin file_path");
        std::process::exit(-1)
    };

    let mut instance = MediaPlayer::new();

    if let Err(e) = instance.initialize() {
        println!("Error occured while initializing pads: {e}");
    }

    let pipeline_weak = instance.pipeline.downgrade();

    instance.decode_bin.connect_pad_added(move |dbin, src_pad| {
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
                    element_warning!(
                        dbin,
                        gstreamer::CoreError::Negotiation,
                        ("Failed to get media type from pad {}", src_pad.name())
                    );

                    return;
                }
                Some(media_type) => media_type,
            }
        };

        let insert_sink = |is_audio, is_video| -> Result<(), Error> {
            if is_audio {
                let queue = gstreamer::ElementFactory::make("queue").build()?;
                let convert = gstreamer::ElementFactory::make("audioconvert").build()?;
                let resample = gstreamer::ElementFactory::make("audioresample").build()?;
                let sink = gstreamer::ElementFactory::make("autoaudiosink").build()?;

                let elements = &[&queue, &convert, &resample, &sink];
                pipeline.add_many(elements)?;
                gstreamer::Element::link_many(elements)?;

                for e in elements {
                    e.sync_state_with_parent()?;
                }

                let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
                src_pad.link(&sink_pad)?;
            } else if is_video {
                let queue = gstreamer::ElementFactory::make("queue").build()?;
                let convert = gstreamer::ElementFactory::make("videoconvert").build()?;
                let scale = gstreamer::ElementFactory::make("videoscale").build()?;
                let sink = gstreamer::ElementFactory::make("autovideosink").build()?;

                let elements = &[&queue, &convert, &scale, &sink];
                pipeline.add_many(elements)?;
                gstreamer::Element::link_many(elements)?;

                for e in elements {
                    e.sync_state_with_parent()?
                }

                let sink_pad = queue.static_pad("sink").expect("queue has no sinkpad");
                src_pad.link(&sink_pad)?;
            }

            Ok(())
        };

        if let Err(err) = insert_sink(is_audio, is_video) {
            element_error!(
                dbin,
                gstreamer::LibraryError::Failed,
                ("Failed to insert sink"),
                details: gstreamer::Structure::builder("error-details")
                            .field("error",
                                   ErrorValue(Arc::new(Mutex::new(Some(err)))))
                            .build()
            );
        }
    });

    instance.start(String::from(uri));

    Ok(())
}

mod launch {
    #[cfg(not(target_os = "macos"))]
    pub fn run<T, F: FnOnce() -> T + Send + 'static>(main: F) -> T
    where
        T: Send + 'static,
    {
        main()
    }

    #[cfg(target_os = "macos")]
    pub fn run<T, F: FnOnce() -> T + Send + 'static>(main: F) -> T
    where
        T: Send + 'static,
    {
        use std::{
            cell::RefCell,
            sync::mpsc::{Sender, channel},
            thread,
        };

        use dispatch::Queue;
        use objc2::rc::Retained;
        use objc2::runtime::ProtocolObject;
        use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
        use objc2_app_kit::{
            NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSEvent,
            NSEventModifierFlags, NSEventSubtype, NSEventType,
        };
        use objc2_foundation::{
            MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint,
        };

        define_class!(
            #[unsafe(super(NSObject))]
            #[thread_kind = MainThreadOnly]
            #[name = "AppDelegate"]
            #[ivars = RefCell<Option<Sender<()>>>]
            struct AppDelegate;

            unsafe impl NSObjectProtocol for AppDelegate {}

            unsafe impl NSApplicationDelegate for AppDelegate {
                #[unsafe(method(applicationDidFinishLaunching:))]
                unsafe fn application_did_finish_launching(&self, _notification: &NSNotification) {
                    if let Some(sender) = self.ivars().borrow_mut().take() {
                        let _ = sender.send(());
                    }
                }
            }
        );

        impl AppDelegate {
            fn new(sender: Sender<()>, mtm: MainThreadMarker) -> Retained<Self> {
                let this = mtm.alloc();
                let this = this.set_ivars(RefCell::new(Some(sender)));
                unsafe { msg_send![super(this), init] }
            }
        }

        let mtm = MainThreadMarker::new().expect("Must be called on main thread");
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

        let (send, recv) = channel::<()>();
        let delegate = AppDelegate::new(send, mtm);
        let delegate = ProtocolObject::from_ref(&*delegate);
        app.setDelegate(Some(delegate));

        let t = thread::spawn(move || {
            // Wait for the NSApp to launch to avoid possibly calling stop_() too early
            recv.recv().unwrap();

            let res = main();

            // Dispatch the stop call to the main queue to be thread-safe
            Queue::main().exec_async(|| {
            // This block runs on the main thread, so MainThreadMarker::new() will succeed
            let mtm = MainThreadMarker::new().expect("Block should run on main thread");
            let app = NSApplication::sharedApplication(mtm);
            app.stop(None);

            // Stopping the event loop requires an actual event
            let location = NSPoint::new(0.0, 0.0);
            let event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
                NSEventType::ApplicationDefined,
                location,
                NSEventModifierFlags::empty(),
                0.0,
                0,
                None,
                NSEventSubtype::ApplicationActivated.0,
                0,
                0,
            ).unwrap();
            app.postEvent_atStart(&event, true);
        });

            res
        });

        app.run();

        t.join().unwrap()
    }
}

fn main() {
    match launch::run(example_main) {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {e}"),
    }
}
