// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::logic::data_types::{playlist::Playlist, state::State};
use std::{cell::RefCell, error::Error, rc::Rc};
use logic::ui_events as ui;
use slint::ComponentHandle;
mod logic;

slint::include_modules!();

struct App {
    window: AppWindow,
    state: Rc<RefCell<State>>,
    runtime: tokio::runtime::Runtime, // Remove if not used, currently planned to be used in online sesarch feature in ui_events file
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let window = AppWindow::new()?;
        let state = Rc::new(RefCell::new(State::default()));
        let runtime = tokio::runtime::Runtime::new()?;

        ui::setup(&window, &state, &runtime);
        Ok(Self { window, state, runtime })
    }

    fn run(self) -> Result<(), Box<dyn Error>> {
        self.window.run()?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
	App::new()?.run()
}
