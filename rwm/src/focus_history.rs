use std::{cmp::Ordering, collections::VecDeque};

use x11rb::protocol::xproto::Window;

use crate::{
    states::{TagState, WinState},
    utils,
};
use common::TagID;

/// A wrapper around a VecDequeue.
/// Currently there is no way to keep a history when switching tags so
/// this might have to change in the future.
#[derive(Debug)]
pub(crate) struct FocusHist {
    windows: VecDeque<WinState>,
    /// The currently focused window in the list, if the list is empty, this is none
    cur: Option<usize>,
}

impl FocusHist {
    pub(crate) fn new() -> Self {
        Self {
            windows: VecDeque::new(),
            cur: None,
        }
    }

    /// An iterator containing all the windows
    pub(crate) fn iter(&self) -> impl Iterator<Item = &WinState> {
        self.windows.iter()
    }

    /// An iterator containing only the windows on the given tags
    pub(crate) fn iter_on_tags(&self, tags: Vec<TagID>) -> impl Iterator<Item = &WinState> {
        self.windows
            .iter()
            .filter(move |&win| tags.iter().any(|tag| win.tags.contains(tag)))
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

    /// "Forget" a window and update self.cur to reflect the change
    pub(crate) fn forget(&mut self, win_id: Window, tags: &[TagState]) {
        let pos = self.windows.iter().position(|win| win.id == win_id);

        if pos.is_none() {
            return;
        }
        let pos = pos.unwrap();

        self.windows.remove(pos);
        if self.windows.is_empty() {
            self.cur = None;
            return;
        }

        if self.cur.is_none() {
            return;
        }
        let cur = self.cur.unwrap();
        // We need to change the position of self.cur or it will be outdated
        match pos.cmp(&cur) {
            Ordering::Less => self.cur = Some(cur - 1),
            Ordering::Greater => { /* We don't need to do anything here */ }
            Ordering::Equal => {
                // Basically what we do in self.focus_next()
                let next = self
                    .windows
                    .iter()
                    .skip(cur)
                    .position(|win| utils::is_visible(win, tags));

                self.cur = match next {
                    Some(v) => Some(v + cur),
                    None => self
                        .windows
                        .iter()
                        .position(|win| utils::is_visible(win, tags)),
                };
            }
        }
    }

    /// Get the Window State and the index of it in the vec
    pub(crate) fn find_by_id(&self, id: Window) -> Option<(usize, &WinState)> {
        self.windows
            .iter()
            .enumerate()
            .find(|(_i, win)| win.id == id)
    }

    /// Get a mutable Window State and the index of it in the vec
    pub(crate) fn find_by_id_mut(&mut self, id: Window) -> Option<(usize, &mut WinState)> {
        self.windows
            .iter_mut()
            .enumerate()
            .find(|(_i, win)| win.id == id)
    }

    /// Get a reference to the focused window
    pub(crate) fn get_focused(&self) -> Option<&WinState> {
        self.cur.map(|cur| &self.windows[cur])
    }

    /// Get a mutable reference to the focused window
    pub(crate) fn get_focused_mut(&mut self) -> Option<&mut WinState> {
        let index = self.cur?;
        Some(&mut self.windows[index])
    }

    /// Find the first visible window in the tags and set it as focused
    pub(crate) fn find_focus(&mut self, tags: &[TagState]) -> Option<usize> {
        self.cur = self
            .windows
            .iter()
            .position(|win| utils::is_visible(win, tags));
        self.cur
    }

    /// Find the next (as in above) visible window
    fn find_next(&self, tags: &[TagState]) -> Option<usize> {
        if let Some(cur) = self.cur {
            let next = self
                .windows
                .iter()
                .skip(cur + 1)
                .position(|win| utils::is_visible(win, tags));

            match next {
                Some(n) => Some(n + cur + 1),
                None => self
                    .windows
                    .iter()
                    .position(|win| utils::is_visible(win, tags)),
            }
        } else {
            self.windows
                .iter()
                .position(|win| utils::is_visible(win, tags))
        }
    }

    /// Find the previous (as in under) visible window
    fn find_prev(&self, tags: &[TagState]) -> Option<usize> {
        let take_rev = |n: usize| self.windows.iter().enumerate().take(n).rev();
        let take_all_rev = || take_rev(self.windows.len());

        self.cur
            .map_or_else(take_all_rev, take_rev)
            .find(|(_, win)| utils::is_visible(win, tags))
            .or_else(|| take_all_rev().find(|(_, win)| utils::is_visible(win, tags)))
            .map(|(i, _)| i)
    }

    /// Give focus to the next window with the correct tags
    pub(crate) fn focus_next(&mut self, tags: &[TagState]) {
        self.cur = self.find_next(tags);
    }

    /// Give focus to the previous window with the correct tags
    pub(crate) fn focus_prev(&mut self, tags: &[TagState]) {
        self.cur = self.find_prev(tags);
    }

    ///  Search for the window with the given id and set it as focused
    pub(crate) fn set_focused(&mut self, id: Window) {
        if let Some((i, _)) = self.find_by_id(id) {
            self.cur = Some(i);
        }
    }
}
