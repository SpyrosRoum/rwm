use std::error::Error;

use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt, StackMode};

use crate::{
    command::{TagSubCommand, WindowSubcommand},
    direction::Direction,
    WMState,
};

impl<'a> WMState<'a> {
    pub(crate) fn on_tag_cmd(&mut self, sub: TagSubCommand) -> Result<(), Box<dyn Error>> {
        match sub {
            TagSubCommand::Toggle(tag) => {
                // I know that it's possible to insert a tag that is already in, but we don't care
                // because it's a hashset so it will be ignored
                if self.tags.contains(&tag) && self.tags.len() > 1 {
                    self.tags.remove(&tag);
                } else {
                    self.tags.insert(tag);
                }

                self.windows.find_focus(&self.tags);
            }
            TagSubCommand::Switch(tag) => {
                self.tags.clear();
                self.tags.insert(tag);

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
                self.windows.focus_next(&self.tags);
                self.update_windows()?;
            }
            WindowSubcommand::Send(tag) => {
                // We want a mutable window state so we get it again as mut
                if let Some(focused_window) = self.windows.get_focused_mut() {
                    focused_window.tags.clear();
                    focused_window.tags.insert(tag);
                }
                self.windows.focus_next(&self.tags);
                self.update_windows()?;
            }
            WindowSubcommand::Focus(dir) => {
                self.on_window_focus(dir)?;
            }
        };

        self.update_windows()?;
        Ok(())
    }

    fn on_window_focus(&mut self, direction: Direction) -> Result<(), Box<dyn Error>> {
        match direction {
            Direction::Up => self.windows.focus_prev(&self.tags),
            Direction::Down => self.windows.focus_next(&self.tags),
        }
        if let Some(win) = self.windows.get_focused() {
            self.conn.configure_window(
                win.id,
                &ConfigureWindowAux::new().stack_mode(StackMode::Above),
            )?;
        }
        Ok(())
    }
}
