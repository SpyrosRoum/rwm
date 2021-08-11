use std::collections::VecDeque;

use x11rb::protocol::xproto::Window;

use crate::states::mon_state::Monitor;
use common::Direction;

/// A data structure that holds the history of monitors
#[derive(Debug)]
pub(crate) struct MonitorsHistory {
    monitors: VecDeque<Monitor>,
    /// The currently focused window in the list, if the list is empty, this is none
    cur: usize,
}

impl MonitorsHistory {
    /// Create a new history, giving focus to the first monitor given
    ///
    /// # Panics
    /// Panics if there is no monitor given
    pub(crate) fn new<M: Into<VecDeque<Monitor>>>(monitors: M) -> Self {
        let monitors = monitors.into();
        assert!(!monitors.is_empty());
        Self { monitors, cur: 0 }
    }

    pub(crate) fn len(&self) -> usize {
        self.monitors.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Monitor> {
        self.monitors.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Monitor> {
        self.monitors.iter_mut()
    }

    pub(crate) fn cur(&self) -> &Monitor {
        &self.monitors[self.cur]
    }

    pub(crate) fn cur_mut(&mut self) -> &mut Monitor {
        &mut self.monitors[self.cur]
    }

    /// Focus the monitor that contains the given window
    ///
    /// # Panics
    /// Will panic if no monitor contains the given window
    pub(crate) fn focus_window(&mut self, window: Window) {
        let new_mon_idx = self
            .monitors
            .iter()
            .enumerate()
            .find(|(_i, m)| m.contains_window(window))
            .map(|(i, _m)| i)
            .expect("Window must be on a monitor");

        self.cur = new_mon_idx;
    }

    /// Focus the monitor that contains the given point, returning a mutable reference to the old monitor
    ///
    /// # Panics:
    /// Will panic if no monitor contains the given point
    pub(crate) fn focus_point(&mut self, x: i16, y: i16) -> &mut Monitor {
        let old = self.cur;

        let new_mon_idx = self
            .monitors
            .iter_mut()
            .enumerate()
            .find(|(_i, mon)| mon.contains_point(x, y))
            .map(|(i, _mon)| i)
            .expect("Can't move outside of monitors");

        self.cur = new_mon_idx;

        &mut self.monitors[old]
    }

    pub(crate) fn focus(&mut self, dir: Direction) {
        self.cur = match dir {
            Direction::Up => {
                if self.cur == 0 {
                    self.monitors.len() - 1
                } else {
                    self.cur - 1
                }
            }
            Direction::Down => {
                if self.cur == self.monitors.len() - 1 {
                    0
                } else {
                    self.cur + 1
                }
            }
        };
        log::debug!("Focusing monitor #{}", self.cur);
    }
}
