use std::error::Error;

use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, StackMode};

use crate::{utils::visible, WMState};
use common::{Direction, TagSubcommand, WindowSubcommand, WindowToggle};

impl<'a> WMState<'a> {
    pub(crate) fn on_tag_cmd(&mut self, sub: TagSubcommand) -> Result<(), Box<dyn Error>> {
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

                self.windows.find_focus(&self.tags);
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

                self.windows.find_focus(&self.tags);
            }
        };

        self.update_windows()?;
        Ok(())
    }

    pub(crate) fn on_window_cmd(&mut self, sub: WindowSubcommand) -> Result<(), Box<dyn Error>> {
        let focused_window = self.windows.get_focused();
        if focused_window.is_none() {
            // there are no windows so just do nothing
            return Ok(());
        }
        let focused_window = focused_window.unwrap();
        match sub {
            WindowSubcommand::Destroy => {
                self.conn.destroy_window(focused_window.id)?;
                self.windows.focus(Direction::Down, &self.tags);
                self.update_windows()?;
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
                self.update_windows()?;
            }
            WindowSubcommand::Focus(dir) => {
                self.on_window_focus(dir)?;
            }
            WindowSubcommand::Toggle(option) => {
                match option {
                    WindowToggle::Float => {
                        if let Some(focused_window) = self.windows.get_focused_mut() {
                            focused_window.floating = !focused_window.floating;
                        }
                    }
                }
                self.update_windows()?;
            }
        };

        self.update_windows()?;
        Ok(())
    }

    fn on_window_focus(&mut self, direction: Direction) -> Result<(), Box<dyn Error>> {
        self.windows.focus(direction, &self.tags);
        if let Some(win) = self.windows.get_focused() {
            self.conn.configure_window(
                win.id,
                &ConfigureWindowAux::new().stack_mode(StackMode::Above),
            )?;
        }
        Ok(())
    }
}
