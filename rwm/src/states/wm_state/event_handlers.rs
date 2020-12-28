use x11rb::{errors::ReplyOrIdError, protocol::xproto::*};

use crate::{utils::clean_mask, WMState};

impl<'a> WMState<'a> {
    pub(crate) fn on_button_press(
        &mut self,
        event: ButtonPressEvent,
    ) -> Result<(), ReplyOrIdError> {
        self.conn.configure_window(
            event.event,
            &ConfigureWindowAux::new().stack_mode(StackMode::Above),
        )?;
        // Left or Right mouse click
        if ![1, 3].contains(&event.detail) {
            return Ok(());
        }

        // We only care if mod key is being pressed as well
        if clean_mask(event.state) != u16::from(self.config.mod_key) {
            return Ok(());
        }

        if let Some((_, mut window)) = self.windows.find_by_id_mut(event.event) {
            window.floating = true;
            if event.detail == 1 {
                // Left click -> Move windows
                let (x, y) = (-event.event_x, -event.event_y);
                self.dragging_window = Some((window.id, (x, y)));
                self.conn.configure_window(
                    window.id,
                    &ConfigureWindowAux::new().border_width(self.config.border_width),
                )?;
            } else {
                // Right click -> Resize window
                let (dst_x, dst_y) = (
                    window.x + window.width as i16,
                    window.y + window.height as i16,
                );
                self.conn
                    .warp_pointer(x11rb::NONE, window.id, 0, 0, 0, 0, dst_x, dst_y)?;
                self.resizing_window = Some((window.id, (dst_x, dst_y)));
            }
        }
        Ok(())
    }

    pub(crate) fn on_motion_notify(
        &mut self,
        event: MotionNotifyEvent,
    ) -> Result<(), ReplyOrIdError> {
        let mut should_update = false;
        if let Some((window, (x, y))) = self.dragging_window {
            if event.event != window {
                return Ok(());
            } else {
                let (x, y) = (x + event.root_x, y + event.root_y);
                let (x, y) = (x as i32, y as i32);
                self.conn
                    .configure_window(window, &ConfigureWindowAux::new().x(x).y(y))?;
                if let Some((_, win_state)) = self.windows.find_by_id_mut(window) {
                    win_state.floating = true;
                    win_state.x = x as i16;
                    win_state.y = y as i16;
                }
            }
            should_update = true;
        }
        if let Some((window, (og_x, og_y))) = self.resizing_window {
            if event.event != window {
                return Ok(());
            } else if let Some((_, win_state)) = self.windows.find_by_id_mut(window) {
                win_state.floating = true;
                let (dif_w, dif_h) = ((event.root_x - og_x) as i32, (event.root_y - og_y) as i32);
                let (new_w, new_h) = (
                    1.max(win_state.width as i32 + dif_w),
                    1.max(win_state.height as i32 + dif_h),
                );
                self.conn.configure_window(
                    window,
                    &ConfigureWindowAux::new()
                        .width(new_w as u32)
                        .height(new_h as u32),
                )?;
                self.resizing_window = Some((window, (event.root_x, event.root_y)));
                win_state.width = new_w as u16;
                win_state.height = new_h as u16;
            }
            should_update = true;
        }

        if should_update {
            self.update_windows()?;
        }
        Ok(())
    }

    pub(crate) fn on_button_release(
        &mut self,
        event: ButtonPressEvent,
    ) -> Result<(), ReplyOrIdError> {
        // Left or Right mouse click
        if ![1, 3].contains(&event.detail) {
            return Ok(());
        }

        if self.dragging_window.is_some() {
            self.dragging_window = None;
        }
        if self.resizing_window.is_some() {
            self.resizing_window = None
        }
        Ok(())
    }

    pub(crate) fn on_enter_notify(
        &mut self,
        event: EnterNotifyEvent,
    ) -> Result<(), ReplyOrIdError> {
        if self.config.follow_cursor {
            self.windows.set_focused(event.event);
            self.update_windows()?;
        }
        Ok(())
    }

    pub(crate) fn on_property_notify(&mut self, event: PropertyNotifyEvent) -> anyhow::Result<()> {
        if event.state == Property::Delete {
            return Ok(());
        }
        if let Some((idx, win_state)) = self.windows.find_by_id(event.window) {
            // Unfortunately I can't use match for event.atom and AtomEnum..
            if event.atom == Atom::from(AtomEnum::WM_TRANSIENT_FOR) {
                let id = self
                    .conn
                    .get_property(
                        false,
                        win_state.id,
                        AtomEnum::WM_TRANSIENT_FOR,
                        AtomEnum::WINDOW,
                        0,
                        1,
                    )?
                    .reply()?
                    .value32()
                    .ok_or_else(|| anyhow::Error::msg("Wrong format"))?
                    .next()
                    .ok_or_else(|| anyhow::Error::msg("No value in reply"))?;
                if self.windows.contains(id) {
                    // We can unwrap because if it didn't exist we wouldn't be here
                    let win_state = self.windows.get_mut(idx).unwrap();
                    win_state.floating = true;
                }
            }
        }

        Ok(())
    }
}
