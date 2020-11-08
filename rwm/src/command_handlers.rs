use std::error::Error;

use x11rb::protocol::xproto::ConnectionExt;

use crate::command::{TagSubCommand, WindowSubcommand};
use crate::WMState;

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
            }
            TagSubCommand::Switch(tag) => {
                self.tags.clear();
                self.tags.insert(tag);
            }
        };

        self.update_windows()?;
        Ok(())
    }

    pub(crate) fn on_window_cmd(&mut self, sub: WindowSubcommand) -> Result<(), Box<dyn Error>> {
        let focused_window = self.get_focused_window();
        if focused_window.is_none() {
            // there are no windows so just do nothing
            return Ok(());
        }

        let focused_window = focused_window.unwrap();

        match sub {
            WindowSubcommand::Destroy => self.conn.destroy_window(focused_window.id)?,
        };

        self.update_windows()?;
        Ok(())
    }
}
