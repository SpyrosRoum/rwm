use std::collections::{vec_deque, HashSet, VecDeque};

use x11rb::protocol::xproto::Window;

use crate::{newtypes::Tag, states::win_state::WinState};

/// A wrapper around a VecDequeue.
/// Currently there is no way to keep a history when switching tags so
/// this might have to change in the future.
#[derive(Debug)]
pub(crate) struct FocusHist {
    windows: VecDeque<WinState>,
    /// The currently focused window in the list, if the list is empty, this is none
    pub(crate) cur: Option<usize>,
}

impl FocusHist {
    pub(crate) fn new() -> Self {
        Self {
            windows: VecDeque::new(),
            cur: None,
        }
    }

    pub(crate) fn iter(&self) -> vec_deque::Iter<'_, WinState> {
        self.windows.iter()
    }

    /// Adds a WinState to front of the history and gives it focus.
    /// If the history did not have this value present, true is returned.
    /// If the history did have this value present, false is returned.
    pub(crate) fn push_front(&mut self, value: WinState) -> bool {
        if let Some(pos) = self.windows.iter().position(|win| win == &value) {
            self.cur = Some(pos);
            false
        } else {
            self.cur = Some(0);
            self.windows.push_front(value);
            true
        }
    }

    pub(crate) fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&WinState) -> bool,
    {
        self.windows.retain(f);
    }

    /// Get the Window State and the index of it in the vec
    pub(crate) fn find_by_id(&self, id: Window) -> Option<(usize, &WinState)> {
        self.windows
            .iter()
            .enumerate()
            .find(|(_i, win)| win.id == id)
    }

    /// Get a reference to the focused window
    pub(crate) fn get_focused(&self) -> Option<&WinState> {
        let index = self.cur?;
        Some(&self.windows[index])
    }

    /// Get a mutable reference to the focused window
    pub(crate) fn get_focused_mut(&mut self) -> Option<&mut WinState> {
        let index = self.cur?;
        Some(&mut self.windows[index])
    }

    /// Find the first window in the tags and set it as focused
    pub(crate) fn find_focus(&mut self, tags: &HashSet<Tag>) -> Option<usize> {
        self.cur = self
            .windows
            .iter()
            .position(|win| tags.iter().any(|tag| win.tags.contains(tag)));
        self.cur
    }

    /// Give focus to the next window with the correct tags
    pub(crate) fn focus_next(&mut self, tags: &HashSet<Tag>) {
        if let Some(index) = self.cur {
            let next = self
                .windows
                .iter()
                .skip(index + 1)
                .position(|win| tags.iter().any(|tag| win.tags.contains(tag)));

            self.cur = match next {
                Some(v) => Some(v + index + 1),
                None => self
                    .windows
                    .iter()
                    .position(|win| tags.iter().any(|tag| win.tags.contains(tag))),
            };
        } else {
            self.cur = self
                .windows
                .iter()
                .position(|win| tags.iter().any(|tag| win.tags.contains(tag)));
        }
    }

    /// Give focus to the previous window with the correct tags
    pub(crate) fn focus_prev(&mut self, tags: &HashSet<Tag>) {
        if let Some(index) = self.cur {
            let prev = self
                .windows
                .iter()
                .enumerate()
                .take(index)
                .rev()
                .find(|(_, win)| tags.iter().any(|tag| win.tags.contains(tag)))
                .map(|(i, _)| i);

            self.cur = match prev {
                Some(v) => Some(v),
                None => self
                    .windows
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, win)| tags.iter().any(|tag| win.tags.contains(tag)))
                    .map(|(i, _)| i),
            };
        } else {
            // If there is nothing focused just focus the last one
            self.cur = self
                .windows
                .iter()
                .enumerate()
                .rev()
                .find(|(_, win)| tags.iter().any(|tag| win.tags.contains(tag)))
                .map(|(i, _)| i);
        }
    }

    pub(crate) fn set_focused(&mut self, id: Window) {
        if let Some((i, _)) = self.find_by_id(id) {
            self.cur = Some(i);
        }
    }
}
