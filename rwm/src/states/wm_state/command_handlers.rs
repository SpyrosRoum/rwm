use {
    anyhow::{Context, Result},
    x11rb::protocol::xproto::ConnectionExt,
};

use crate::{utils::visible, WmState};
use common::{Direction, TagSubcommand, WindowSubcommand, WindowToggle};

impl<'a> WmState<'a> {
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
                return self.on_window_cmd(WindowSubcommand::Focus(Direction::Down));
            }
            WindowSubcommand::Send { tag_id } => {
                // We want a mutable window state so we get it again as mut and we know it exists
                let focused_window = self.windows.get_focused_mut().unwrap();
                let tag_state = self.tags.iter().find(|tag_state| **tag_state == tag_id);
                let tag = match tag_state {
                    Some(t) => t.id,
                    None => tag_id,
                };
                focused_window.tags.clear();
                focused_window.tags.insert(tag);

                if let Some((_, new_focused)) = self.windows.find_next(&self.tags) {
                    let id = new_focused.id;
                    self.focus(id)?;
                }
            }
            WindowSubcommand::Focus(dir) => {
                let new_focused = match dir {
                    Direction::Up => self
                        .windows
                        .find_prev(&self.tags)
                        .map(|(_, new_focused)| new_focused),
                    Direction::Down => self
                        .windows
                        .find_next(&self.tags)
                        .map(|(_, new_focused)| new_focused),
                };

                if let Some(new_focused) = new_focused {
                    let id = new_focused.id;
                    self.focus(id)?;
                }
            }
            WindowSubcommand::Shift(dir) => {
                self.windows.shift(dir, &self.tags);
                return self.on_window_cmd(WindowSubcommand::Focus(dir));
            }
            WindowSubcommand::Toggle(option) => match option {
                WindowToggle::Float => {
                    if let Some(focused_window) = self.windows.get_focused_mut() {
                        focused_window.floating = !focused_window.floating;
                    }
                }
            },
        };

        self.update_windows().context(format!(
            "Failed to update windows after `Window({:?})`",
            sub
        ))
    }
}
