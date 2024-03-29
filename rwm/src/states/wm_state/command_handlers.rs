use {
    anyhow::{Context, Result},
    x11rb::protocol::xproto::ConnectionExt,
};

use crate::{utils::visible, WmState};
use common::{
    Command, Destination, Direction, MonitorSubcommand, TagSubcommand, WindowSubcommand,
    WindowToggle,
};

impl<'a> WmState<'a> {
    pub(crate) fn on_tag_cmd(&mut self, sub: TagSubcommand) -> Result<()> {
        match sub {
            TagSubcommand::Toggle { tag_id } => {
                let one_vis = visible(&self.monitors.cur().tags).len() == 1;
                if let Some(mut tag_state) = self
                    .monitors
                    .cur_mut()
                    .tags
                    .iter_mut()
                    .find(|tag_state| **tag_state == tag_id)
                {
                    if tag_state.visible && one_vis {
                        // There is only one visible tag so we can't make that invisible too
                        return Ok(());
                    }
                    tag_state.visible = !tag_state.visible;
                }
            }
            TagSubcommand::Switch { tag_id } => {
                self.monitors.cur_mut().switch_tag(tag_id);
            }
        };
        self.monitors.cur_mut().reset_focus();

        self.update_windows()
            .with_context(|| format!("Failed to update windows after `Tag({:?})`", sub))
    }

    pub(crate) fn on_window_cmd(&mut self, sub: WindowSubcommand) -> Result<()> {
        let focused_window = self.monitors.cur().windows.get_focused();
        if focused_window.is_none() {
            // There is no focused window so just do nothing
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
            WindowSubcommand::Send(Destination::Tag { tag_id }) => {
                // We want a mutable window state so we get it again as mut and we know it exists
                let tag_state = self
                    .monitors
                    .cur()
                    .tags
                    .iter()
                    .find(|tag_state| **tag_state == tag_id);
                let tag = match tag_state {
                    Some(t) => t.id,
                    None => tag_id,
                };
                let focused_window = self.monitors.cur_mut().windows.get_focused_mut().unwrap();
                focused_window.tags.clear();
                focused_window.tags.insert(tag);

                if let Some(new_focused) = self.monitors.cur().get_next_win() {
                    let id = new_focused.id;
                    self.focus(id)?;
                }
            }
            WindowSubcommand::Send(Destination::Monitor(dir)) => {
                if self.monitors.len() > 1 {
                    let cur = self.monitors.cur_mut();
                    let focused_win = cur.windows.get_focused();
                    if focused_win.is_none() {
                        // We just need to shift focus
                        self.handle_command(Command::Monitor(MonitorSubcommand::Focus(dir)))?;
                        return Ok(());
                    }
                    let focused_win = focused_win.unwrap().id;

                    let (win, new) = cur.forget(focused_win);
                    if let Some(new) = new {
                        let id = new.id;
                        cur.windows.set_focused(id);
                    }
                    let win = win.expect("It certainly exists");
                    let id = win.id;

                    self.monitors.focus(dir);
                    self.monitors.cur_mut().windows.push_front(win);
                    self.monitors.cur_mut().windows.set_focused(id);

                    self.focus(id)?;
                }
            }
            WindowSubcommand::Focus(dir) => {
                let new_focused = match dir {
                    Direction::Up => self.monitors.cur().get_prev_win(),
                    Direction::Down => self.monitors.cur().get_next_win(),
                };

                if let Some(new_focused) = new_focused {
                    let id = new_focused.id;
                    self.focus(id)?;
                }
            }
            WindowSubcommand::Shift(dir) => {
                self.monitors.cur_mut().shift_windows(dir);
                return self.on_window_cmd(WindowSubcommand::Focus(dir));
            }
            WindowSubcommand::Toggle(option) => match option {
                WindowToggle::Float => {
                    if let Some(focused_window) = self.monitors.cur_mut().windows.get_focused_mut()
                    {
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
