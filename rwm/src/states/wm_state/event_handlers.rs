use x11rb::{connection::Connection, errors::ReplyOrIdError, protocol::xproto::*};

use crate::{utils::clean_mask, utils::get_transient_for, WmState};

impl<'a> WmState<'a> {
    pub(crate) fn on_button_press(
        &mut self,
        event: ButtonPressEvent,
    ) -> Result<(), ReplyOrIdError> {
        log::info!("Handling {:?}", event);
        let screen = &self.conn.setup().roots[self.screen_num];
        if event.event == screen.root {
            return Ok(());
        }

        self.focus(event.event)?;

        // Left or Right mouse click
        if ![1, 3].contains(&event.detail) {
            return Ok(());
        }

        // We only care if mod key is being pressed as well
        if clean_mask(event.state) != u16::from(self.config.mod_key) {
            return Ok(());
        }

        // We handle changing `self.cur_monitor` in `motion_notify` so we can assume that the mouse
        // is in the currently focused monitor
        if let Some((_, mut window)) = self.monitors.cur_mut().windows.find_by_id_mut(event.event) {
            window.floating = true;
            if event.detail == 1 && self.resizing_window.is_none() {
                // Left click -> Move windows
                let (x, y) = (-event.event_x, -event.event_y);
                self.dragging_window = Some((window.id, (x, y)));
                // We set the border_width in case it was previously the only window and it didn't have a border
                self.conn.configure_window(
                    window.id,
                    &ConfigureWindowAux::new().border_width(self.config.border_width),
                )?;
                self.set_cursor(event.event, "fleur")?;
            } else if event.detail == 3 && self.dragging_window.is_none() {
                // Right click -> Resize window
                // dst_x and dst_y in warp_pointer is the offset from the window origin
                let (dst_x, dst_y) = (window.width as i16, window.height as i16);
                self.conn
                    .warp_pointer(x11rb::NONE, window.id, 0, 0, 0, 0, dst_x, dst_y)?;
                self.resizing_window = Some((window.id, (dst_x + window.x, dst_y + window.y)));
                self.set_cursor(event.event, "bottom_right_corner")?;
            }
        }
        Ok(())
    }

    pub(crate) fn on_motion_notify(
        &mut self,
        event: MotionNotifyEvent,
    ) -> Result<(), ReplyOrIdError> {
        log::info!("Handling {:?}", event);
        let mut should_update = false;

        // Changed focused monitor only if:
        //   - The point is not in the current monitor
        //   - AND we:
        //       - Are dragging a window
        //       - OR Resizing a window
        //       - OR we have config.follow_cursor to true
        if !self
            .monitors
            .cur()
            .contains_point(event.root_x, event.root_y)
            && ((self.dragging_window.is_some() || self.resizing_window.is_some())
                || self.config.follow_cursor)
        {
            should_update = true;
            let old = self.monitors.focus_point(event.root_x, event.root_y);

            if let Some((win_id, _)) = self.dragging_window {
                let (win, new) = old.forget(win_id);
                if let Some(new) = new {
                    let id = new.id;
                    old.windows.set_focused(id);
                }
                let win = win.expect("It certainly exists");
                let id = win.id;

                self.monitors.cur_mut().windows.push_front(win);
                self.monitors.cur_mut().windows.set_focused(id);
            }
        }

        if let Some((window, (x, y))) = self.dragging_window {
            if event.event != window {
                return Ok(());
            } else {
                let (x, y) = (x + event.root_x, y + event.root_y);
                let (x, y) = (x as i32, y as i32);
                self.conn.configure_window(
                    window,
                    &ConfigureWindowAux::new()
                        .x(x)
                        .y(y)
                        .stack_mode(StackMode::ABOVE),
                )?;
                if let Some((_, win_state)) = self.monitors.cur_mut().windows.find_by_id_mut(window)
                {
                    win_state.floating = true;
                    win_state.x = x as i16;
                    win_state.y = y as i16;
                }
            }
            should_update = true;
        } else if let Some((window, (og_x, og_y))) = self.resizing_window {
            if event.event != window {
                return Ok(());
            } else if let Some((_, win_state)) =
                self.monitors.cur_mut().windows.find_by_id_mut(window)
            {
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
                        .height(new_h as u32)
                        .stack_mode(StackMode::ABOVE),
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
        log::info!("Handling {:?}", event);
        // Left or Right mouse click
        if ![1, 3].contains(&event.detail) {
            return Ok(());
        }

        if event.detail == 1 {
            if let Some((window, _)) = self.dragging_window {
                self.set_cursor(window, "left_ptr")?;
                self.dragging_window = None;
            }
        } else if let Some((window, _)) = self.resizing_window {
            self.set_cursor(window, "left_ptr")?;
            self.resizing_window = None
        }

        Ok(())
    }

    pub(crate) fn on_enter_notify(
        &mut self,
        event: EnterNotifyEvent,
    ) -> Result<(), ReplyOrIdError> {
        log::info!("Handling {:?}", event);
        if event.mode != NotifyMode::NORMAL || event.detail != NotifyDetail::INFERIOR {
            // If we use a command to shift the window and a new window comes under our cursor,
            // we get NONLINEAR enter notify, so we ignore it
            return Ok(());
        }

        if self.config.follow_cursor {
            self.focus(event.event)?;
        }
        Ok(())
    }

    pub(crate) fn on_property_notify(&mut self, event: PropertyNotifyEvent) -> anyhow::Result<()> {
        log::info!("Handling {:?}", event);
        if event.state == Property::DELETE {
            return Ok(());
        }

        let win = self
            .iter_windows()
            .enumerate()
            .find(|(_i, w)| w.id == event.window);
        if win.is_none() {
            return Ok(());
        }
        let (index, win_state) = win.unwrap();

        // Unfortunately I can't use match for event.atom and AtomEnum..
        if event.atom == Atom::from(AtomEnum::WM_TRANSIENT_FOR) {
            let id = get_transient_for(self.conn, win_state.id)?;
            if id.is_none() {
                // That's okay!
                return Ok(());
            }
            let id = id.unwrap();
            if self.iter_windows().any(|win| win.id == id) {
                // We can unwrap because if it didn't exist we wouldn't be here
                let win_state = self.iter_windows_mut().nth(index).unwrap();
                win_state.floating = true;
            }
        }

        Ok(())
    }
}
