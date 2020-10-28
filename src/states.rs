use x11rb::connection::Connection;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

use crate::config::Config;
use crate::utils::clean_mask;

#[derive(Debug)]
pub struct WinState {
    pub(crate) id: Window,
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

#[derive(Debug)]
pub struct WMState<'a, C: Connection> {
    pub(crate) conn: &'a C,
    pub(crate) config: Config,
    pub(crate) screen_num: usize,
    pub(crate) windows: Vec<WinState>,
    // If this is Some, we are currently dragging the given window with the given offset relative
    // to the mouse.
    pub(crate) selected_window: Option<(Window, (i16, i16))>,
}

impl WinState {
    pub fn new(win: Window, geom: &GetGeometryReply) -> Self {
        Self {
            id: win,
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: geom.height,
        }
    }
}

impl<'a, C> WMState<'a, C>
where
    C: Connection,
{
    pub fn new(conn: &'a C, screen_num: usize, config: Config) -> Self {
        Self {
            conn,
            config,
            screen_num,
            windows: vec![],
            selected_window: None,
        }
    }

    fn find_window_by_id(&self, id: Window) -> Option<&WinState> {
        self.windows.iter().find(|win| win.id == id)
    }

    /// Scan for pre-existing windows and manage them
    pub fn scan_windows(&mut self) -> Result<(), ReplyOrIdError> {
        let screen = &self.conn.setup().roots[self.screen_num];
        let tree = self.conn.query_tree(screen.root)?.reply()?;

        // Bunch request the attributes of each window
        let mut cookies = Vec::with_capacity(tree.children.len());
        for win in tree.children {
            let attr = self.conn.get_window_attributes(win)?;
            cookies.push((win, attr));
        }

        // Get the replies and manage the windows
        for (win, attr) in cookies {
            let attr = attr.reply();
            if attr.is_err() {
                // Just skip this window
                continue;
            }
            let attr = attr.unwrap();
            if !attr.override_redirect && attr.map_state != MapState::Unmapped {
                self.manage_window(win)?;
            }
        }

        Ok(())
    }

    pub fn manage_window(&mut self, window: Window) -> Result<(), ReplyOrIdError> {
        // Add a border
        let config = ConfigureWindowAux::default().border_width(self.config.border_width);
        self.conn.configure_window(window, &config)?;

        // Give color to the border
        let attrs = ChangeWindowAttributesAux::default().border_pixel(self.config.border_color);
        self.conn.change_window_attributes(window, &attrs)?;

        // Register the proper events with the window
        let events = ChangeWindowAttributesAux::default().event_mask(
            EventMask::KeyPress
                | EventMask::KeyRelease
                | EventMask::ButtonRelease
                | EventMask::PointerMotion
                | EventMask::EnterWindow,
        );
        self.conn
            .change_window_attributes(window, &events)?
            .check()?;

        // Get Button Press events
        // This ugly line is needed because grab_button expects something that implements Into<u16>
        // but EventMask is u32
        let event_mask =
            u32::from(EventMask::ButtonPress | EventMask::ButtonRelease | EventMask::PointerMotion)
                as u16;
        self.conn.grab_button(
            false,
            window,
            event_mask,
            GrabMode::Async,
            GrabMode::Async,
            x11rb::NONE,
            x11rb::NONE,
            ButtonIndex::Any,
            ModMask::Any,
        )?;

        // Show the window
        self.conn.map_window(window)?;

        let geom = self.conn.get_geometry(window)?.reply()?;
        self.windows.push(WinState::new(window, &geom));

        Ok(())
    }

    pub fn handle_event(&mut self, event: Event) -> Result<(), ReplyOrIdError> {
        match event {
            Event::MapRequest(event) => self.manage_window(event.window)?,
            Event::ButtonPress(event) => self.handle_button_press(event)?,
            Event::ButtonRelease(event) => self.handle_button_release(event)?,
            Event::MotionNotify(event) => self.handle_motion_notify(event)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_button_press(&mut self, event: ButtonPressEvent) -> Result<(), ReplyOrIdError> {
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
            return Ok(())
        }

        if let Some(window) = self.find_window_by_id(event.event) {
            let (x, y) = (-event.event_x, -event.event_y);
            self.selected_window = Some((window.id, (x, y)));
        }
        Ok(())
    }

    fn handle_motion_notify(&mut self, event: MotionNotifyEvent) -> Result<(), ReplyOrIdError> {
        if let Some((window, (x, y))) = self.selected_window {
            if event.event != window {
                return Ok(());
            } else {
                let (x, y) = (x + event.root_x, y + event.root_y);
                let (x, y) = (x as i32, y as i32);
                self.conn
                    .configure_window(window, &ConfigureWindowAux::new().x(x).y(y))?;
            }
        }
        Ok(())
    }

    fn handle_button_release(&mut self, event: ButtonPressEvent) -> Result<(), ReplyOrIdError> {
        // Left mouse click
        if event.detail != 1 {
            return Ok(());
        }

        if let Some((window, (x, y))) = self.selected_window {
            if window == event.event {
                self.selected_window = None;
            }
        }
        Ok(())
    }
}
