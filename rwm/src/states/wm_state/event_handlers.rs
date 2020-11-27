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
        // Left mouse click
        if event.detail != 1 {
            return Ok(());
        }

        // We only care if mod key is being pressed as well
        if clean_mask(event.state) != self.config.mod_key.into() {
            return Ok(());
        }

        if let Some((_, mut window)) = self.windows.find_by_id_mut(event.event) {
            let (x, y) = (-event.event_x, -event.event_y);
            self.selected_window = Some((window.id, (x, y)));
            window.floating = true;
            self.conn.configure_window(
                window.id,
                &ConfigureWindowAux::new().border_width(self.config.border_width),
            )?;
        }
        Ok(())
    }

    pub(crate) fn on_motion_notify(
        &mut self,
        event: MotionNotifyEvent,
    ) -> Result<(), ReplyOrIdError> {
        if let Some((window, (x, y))) = self.selected_window {
            if event.event != window {
                return Ok(());
            } else {
                let (x, y) = (x + event.root_x, y + event.root_y);
                let (x, y) = (x as i32, y as i32);
                self.conn
                    .configure_window(window, &ConfigureWindowAux::new().x(x).y(y))?;
            }
            self.update_windows()?;
        }
        Ok(())
    }

    pub(crate) fn on_button_release(
        &mut self,
        event: ButtonPressEvent,
    ) -> Result<(), ReplyOrIdError> {
        // Left mouse click
        if event.detail != 1 {
            return Ok(());
        }

        if let Some((window, _)) = self.selected_window {
            if window == event.event {
                self.selected_window = None;
            }
        }
        Ok(())
    }

    pub(crate) fn on_focus_in(&mut self, event: FocusInEvent) -> Result<(), ReplyOrIdError> {
        self.windows.set_focused(event.event);
        self.update_windows()?;
        Ok(())
    }

    pub(crate) fn on_enter_notify(
        &mut self,
        event: EnterNotifyEvent,
    ) -> Result<(), ReplyOrIdError> {
        self.windows.set_focused(event.event);
        self.update_windows()?;
        Ok(())
    }
}
