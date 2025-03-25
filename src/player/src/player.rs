use std::{
    env,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use gst::prelude::*;

#[allow(unused_imports)]
#[path = "./common.rs"]
mod common;

pub fn main_loop(uri: &str) -> Result<(), Error> {
    gst::init()?;
    let main_loop = glib::MainLoop::new(None, false);

    let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
    let player = gst_player::Player::new(
        None::<gst_player::PlayerVideoRenderer>,
        Some(dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
    );

    player.set_uri(Some(uri));
    let error = Arc::new(Mutex::new(Ok(())));

    let main_loop_clone = main_loop.clone();
    // Connect to the player's "end-of-stream" signal, which will tell us when the
    // currently played media stream reached its end.
    // By default the player is killed at the end.
    player.connect_end_of_stream(move |player| {
        let main_loop = &main_loop_clone;
        player.stop();
        main_loop.quit();
    });

    let main_loop_clone = main_loop.clone();
    let error_clone = Arc::clone(&error);
    // Connect to the player's "error" signal, which will inform us about eventual
    // errors (such as failing to retrieve a http stream).
    // By default the player is killed.
    player.connect_error(move |player, err| {
        let main_loop = &main_loop_clone;
        let error = &error_clone;

        *error.lock().unwrap() = Err(err.clone());

        player.stop();
        main_loop.quit();
    });

    player.play();
    main_loop.run();

    let guard = error.as_ref().lock().unwrap();

    guard.clone().map_err(|e| e.into())
}

#[allow(unused_variables)]
fn example_main() {
    let args: Vec<_> = env::args().collect();
    let uri: &str = if args.len() == 2 {
        args[1].as_ref()
    } else {
        println!("Usage: player uri");
        std::process::exit(-1)
    };

    match main_loop(uri) {
        Ok(r) => r,
        Err(e) => eprintln!("Error! {e}"),
    }
}

fn main() {
    // tutorials_common::run is only required to set up the application environment on macOS
    // (but not necessary in normal Cocoa applications where this is set up automatically)
    common::run(example_main);
}
