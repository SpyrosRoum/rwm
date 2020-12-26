use x11rb::protocol::xproto::ConnectionExt;

use crate::{utils::visible, WMState};
use anyhow::{Context, Result};
use common::{Direction, TagSubcommand, WindowSubcommand, WindowToggle};

impl<'a> WMState<'a> {
    pub(crate) fn on_tag_cmd(&mut self, sub: TagSubcommand) -> Result<()> {
        match sub {
            TagSubcommand::Toggle { tag_id } => {
                let one_vis = visible(&self.tags).len() == 1;
                if let Some(mut tag_state) =
                    self.tags.iter_mut().find(|tag_state| **tag_state == tag_id)
                {
                    if tag_state.visible && one_vis {
                        return Ok(());
                    }
                    tag_state.visible = !tag_state.visible;
                }

                self.windows.reset_focus(&self.tags);
            }
            TagSubcommand::Switch { tag_id } => {
                for tag_state in self.tags.iter_mut() {
                    if *tag_state == tag_id {
                        tag_state.visible = true;
                        self.layout = tag_state.layout;
                    } else {
                        tag_state.visible = false;
                    }
                }

                self.windows.reset_focus(&self.tags);
            }
        };

        self.update_windows()
            .with_context(|| format!("Failed to update windows after `Tag({:?})`", sub))
    }

    pub(crate) fn on_window_cmd(&mut self, sub: WindowSubcommand) -> Result<()> {
        let focused_window = self.windows.get_focused();
        if focused_window.is_none() {
            // there are no windows so just do nothing
            return Ok(());
        }
        let focused_window = focused_window.unwrap();
        match sub {
            WindowSubcommand::Destroy => {
                self.conn
                    .destroy_window(focused_window.id)
                    .context("Failed to destroy the current window")?;
                self.windows.focus(Direction::Down, &self.tags);
            }
            WindowSubcommand::Send { tag_id } => {
                // We want a mutable window state so we get it again as mut
                if let Some(focused_window) = self.windows.get_focused_mut() {
                    let tag_state = self.tags.iter().find(|tag_state| **tag_state == tag_id);
                    let tag = match tag_state {
                        Some(t) => t.id,
                        None => tag_id,
                    };
                    focused_window.tags.clear();
                    focused_window.tags.insert(tag);
                }
                self.windows.focus(Direction::Down, &self.tags);
            }
            WindowSubcommand::Focus(dir) => self.windows.focus(dir, &self.tags),
            WindowSubcommand::Shift(dir) => {
                if let Some(id) = self.windows.get_focused().map(|win| win.id) {
                    self.windows.shift(dir, &self.tags);
                    self.windows.set_focused(id)
                }
            }
            WindowSubcommand::Toggle(option) => match option {
                WindowToggle::Float => {
                    if let Some(focused_window) = self.windows.get_focused_mut() {
                        focused_window.floating = !focused_window.floating;
                    }
                }
            },
        };

        self.update_windows()
            .with_context(|| format!("Failed to update windows after `Window({:?})`", sub))
    }
}
