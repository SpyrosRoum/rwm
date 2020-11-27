use std::{
    cmp::Ordering,
    collections::{vec_deque, VecDeque},
};

use x11rb::protocol::xproto::Window;

use crate::{
    states::{TagState, WinState},
    tag_id::TagID,
};

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

    pub(crate) fn iter(&self) -> vec_deque::Iter<'_, WinState> {
        self.windows.iter()
    }

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
                let next = self.windows.iter().skip(cur).position(|win| {
                    tags.iter()
                        .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
                });

                self.cur = match next {
                    Some(v) => Some(v + cur),
                    None => self.windows.iter().position(|win| {
                        tags.iter()
                            .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
                    }),
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
    pub(crate) fn find_focus(&mut self, tags: &[TagState]) -> Option<usize> {
        self.cur = self.windows.iter().position(|win| {
            tags.iter()
                .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
        });
        self.cur
    }

    /// Give focus to the next window with the correct tags
    pub(crate) fn focus_next(&mut self, tags: &[TagState]) {
        if let Some(index) = self.cur {
            let next = self.windows.iter().skip(index + 1).position(|win| {
                tags.iter()
                    .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
            });

            self.cur = match next {
                Some(v) => Some(v + index + 1),
                None => self.windows.iter().position(|win| {
                    tags.iter()
                        .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
                }),
            };
        } else {
            self.cur = self.windows.iter().position(|win| {
                tags.iter()
                    .any(|&tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
            });
        }
    }

    /// Give focus to the previous window with the correct tags
    pub(crate) fn focus_prev(&mut self, tags: &[TagState]) {
        if let Some(index) = self.cur {
            let prev = self
                .windows
                .iter()
                .enumerate()
                .take(index)
                .rev()
                .find(|(_, win)| {
                    tags.iter()
                        .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
                })
                .map(|(i, _)| i);

            self.cur = match prev {
                Some(v) => Some(v),
                None => self
                    .windows
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, win)| {
                        tags.iter()
                            .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
                    })
                    .map(|(i, _)| i),
            };
        } else {
            // If there is nothing focused just focus the last one
            self.cur = self
                .windows
                .iter()
                .enumerate()
                .rev()
                .find(|(_, win)| {
                    tags.iter()
                        .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
                })
                .map(|(i, _)| i);
        }
    }

    pub(crate) fn set_focused(&mut self, id: Window) {
        if let Some((i, _)) = self.find_by_id(id) {
            self.cur = Some(i);
        }
    }
}