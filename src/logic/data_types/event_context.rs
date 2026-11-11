use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};
use crate::{AppWindow, SlintState, logic::{audio::media_player::MediaPlayer, data_types::state::State}};

#[derive(Clone)]
pub struct EventContext {
    pub app: slint::Weak<AppWindow>,
    pub state: Rc<RefCell<State>>,
    pub player: Rc<RefCell<MediaPlayer>>,
}

impl EventContext {
    pub fn with_state<R>(&self, f: impl FnOnce(&mut State, &SlintState,  &Rc<RefCell<MediaPlayer>>) -> R) -> Option<R> {
        let app = self.app.upgrade()?;
        let global = app.global::<SlintState>();
        Some(f(&mut self.state.borrow_mut(), &global, &self.player))
    }
}
