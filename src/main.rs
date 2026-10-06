// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::logic::data_types::{playlist::Playlist, state::State};
use std::{cell::RefCell, error::Error, rc::Rc};
use logic::ui_events as ui;
use slint::ComponentHandle;
mod logic;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = AppWindow::new()?;
    let mut state = State::default();
    let tokio_runtime = tokio::runtime::Runtime::new()?;

    ui::handle_initialization(&mut state);
    ui::handle_passing_values(&app, &mut state);
    ui::handle_events(&app, &mut Rc::new(RefCell::new(state)), &tokio_runtime);
    app.run()?;

    Ok(())
}
