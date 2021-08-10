use std::{cmp::Ordering, collections::VecDeque};

use x11rb::protocol::xproto::Window;

use crate::{
    states::{TagState, WinState},
    utils,
};
use common::{Direction, TagId};

/// A wrapper around a VecDequeue.
/// Currently there is no way to keep a history when switching tags so
/// this might have to change in the future.
#[derive(Debug)]
pub(crate) struct WindowsHistory {
    windows: VecDeque<WinState>,
    /// The currently focused window in the list, if the list is empty, this is none
    cur: Option<usize>,
}

impl WindowsHistory {
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

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut WinState> {
        self.windows.iter_mut()
    }

    pub(crate) fn contains<T: Into<Window>>(&self, id: T) -> bool {
        let id: Window = id.into();
        self.windows.iter().any(|win_state| win_state.id == id)
    }

    /// An iterator containing only the windows on the given tags
    pub(crate) fn iter_on_tags_mut(
        &mut self,
        tags: Vec<TagId>,
    ) -> impl Iterator<Item = &mut WinState> {
        self.windows
            .iter_mut()
            .filter(move |win| tags.iter().any(|tag| win.tags.contains(tag)))
    }

    /// Adds a WinState to front of the history without giving it focus.
    pub(crate) fn push_front(&mut self, value: WinState) {
        self.windows.push_front(value);
        if let Some(cur) = self.cur {
            self.cur = Some(cur + 1);
        }
    }

    /// "Forget" a window returning it and the next window that should get focus
    /// Warning: self.cur points to None after this
    pub(crate) fn forget(
        &mut self,
        win_id: Window,
        tags: &[TagState],
    ) -> (Option<WinState>, Option<&WinState>) {
        let pos = self.windows.iter().position(|win| win.id == win_id);
        if pos.is_none() {
            return (None, None);
        }
        let pos = pos.unwrap();

        let cur = self
            .cur
            .expect("There is at least one window so it has focus");
        self.cur = None;

        let win = self.windows.remove(pos);
        if self.windows.is_empty() {
            return (win, None);
        }

        let new = match pos.cmp(&cur) {
            Ordering::Less => Some(&self.windows[cur - 1]),
            Ordering::Greater => Some(&self.windows[cur]),
            Ordering::Equal => {
                // Basically what we do in `self.find_next()`
                self.windows
                    .iter()
                    .skip(cur)
                    .chain(self.windows.iter())
                    .find(|win| utils::is_visible(win, tags))
            }
        };

        (win, new)
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
    // Note: We handle the change of tags on the command so the window returned should always be visible
    pub(crate) fn get_focused(&self) -> Option<&WinState> {
        self.cur.map(|cur| &self.windows[cur])
    }

    /// Get a mutable reference to the focused window
    pub(crate) fn get_focused_mut(&mut self) -> Option<&mut WinState> {
        let index = self.cur?;
        Some(&mut self.windows[index])
    }

    /// Find the first visible window in the tags and set it as focused
    pub(crate) fn reset_focus(&mut self, tags: &[TagState]) -> Option<usize> {
        self.cur = self
            .windows
            .iter()
            .position(|win| utils::is_visible(win, tags));
        self.cur
    }

    /// Find the next (as in Direction::Down) visible window
    pub(crate) fn find_next(&self, tags: &[TagState]) -> Option<(usize, &WinState)> {
        let find_first = || {
            self.windows
                .iter()
                .enumerate()
                .find(|(_, win)| utils::is_visible(win, tags))
        };

        if let Some(cur) = self.cur {
            let next = self
                .windows
                .iter()
                .enumerate()
                .skip(cur + 1)
                .find(|(_, win)| utils::is_visible(win, tags));

            next.or_else(find_first)
        } else {
            find_first()
        }
    }

    /// Find the previous (as in Direction::Up) visible window
    pub(crate) fn find_prev(&self, tags: &[TagState]) -> Option<(usize, &WinState)> {
        let take_rev = |n: usize| self.windows.iter().enumerate().take(n).rev();
        let take_all_rev = || take_rev(self.windows.len());

        self.cur
            .map_or_else(take_all_rev, take_rev)
            .find(|(_, win)| utils::is_visible(win, tags))
            .or_else(|| take_all_rev().find(|(_, win)| utils::is_visible(win, tags)))
    }

    /// Move the current window up or down, focus doesn't follow
    pub(crate) fn shift(&mut self, dir: Direction, tags: &[TagState]) {
        if let Some(cur) = self.cur {
            let next_pos = match dir {
                Direction::Up => self.find_prev(tags).map(|(i, _)| i),
                Direction::Down => self.find_next(tags).map(|(i, _)| i),
            }
            .expect("There is at least one window");
            self.windows.swap(cur, next_pos);
        }
    }

    ///  Search for the window with the given id and set it as focused
    pub(crate) fn set_focused(&mut self, id: Window) {
        if let Some((i, _)) = self.find_by_id(id) {
            self.cur = Some(i);
        }
    }
}
