use std::error::Error;

use crate::command::TagSubCommand;
use crate::WMState;

impl<'a> WMState<'a> {
    pub(crate) fn handle_tag_cmd(&mut self, sub: TagSubCommand) -> Result<(), Box<dyn Error>> {
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
}
