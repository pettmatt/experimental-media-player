use crate::{SlintState, SlintTimeline, logic::{convert::format_into_time, data_types::{ToSlint, track::Track}}, slint_generatedAppWindow};
use std::{cell::RefCell, rc::Rc};
use slint::SharedString;

#[derive(Clone, Debug, Default)]
pub struct Timeline {
    pub current: i32,
    pub length: i32,
    pub media_index: Option<usize>,
    pub queue_index: Option<usize>,
}

impl Timeline {
	pub fn set_timeline(&mut self, track: Rc<RefCell<Track>>, globals: SlintState) {
        self.length = track.borrow().duration;
        self.current = 0;
        self.media_index = Some(track.borrow().id as usize);
        let timeline: SlintTimeline = self.to_slint(None);
        globals.set_timeline(timeline);
    }

    pub fn update_timeline(&mut self, value: i32, globals: SlintState) {
        self.current = value;
        let timeline: SlintTimeline = self.to_slint(None);
        globals.set_timeline(timeline);
    }
}

impl ToSlint for Timeline {
	type Output = slint_generatedAppWindow::SlintTimeline;
	type Overrides = Option<String>;

	fn to_slint(&self, _overrides: Self::Overrides) -> Self::Output {
		Self::Output {
			current: self.current,
			length: self.length,
			str_current: SharedString::from(format_into_time(self.current as f64)),
			str_length: SharedString::from(format_into_time(self.length as f64)),
		}
	}

	fn to_slint_empty() -> Self::Output {
		Self::Output {
			current: 0,
			length: 0,
			str_current: SharedString::from(format_into_time(0 as f64)),
			str_length: SharedString::from(format_into_time(0 as f64)),
        }
	}
}
