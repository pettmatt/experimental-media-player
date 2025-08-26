// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fmt, i32};
use logic::ui;

mod logic;

slint::include_modules!();

struct Settings {}
struct Authentication {}

// struct State {
// 	playing: Result<None, fmt::Error>,
// 	queue: Vec<String>,
// 	playlists: Vec<String>,
// 	sources: Vec<String>,
// 	authentications: Vec<Authentication>,
// 	settings: Settings
// }

fn main() -> Result<(), Box<dyn Error>> {
	// let state: State; // Will hold the main state of Slint
    let app = AppWindow::new()?;

    // app.on_request_increase_value({
    //     let ui_handle = app.as_weak();
    //     move || {
    //         let ui = ui_handle.unwrap();
    //         ui.set_counter(ui.get_counter() + 1);
    //     }
    // });

	ui::handle_events(&app);

    app.run()?;

    Ok(())
}
