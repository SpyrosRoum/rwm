use std::collections::HashSet;
use std::io::Read;
use std::os::unix::net::UnixStream;

use x11rb::connection::Connection;
use x11rb::errors::ReplyOrIdError;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use crate::config::Config;
use std::error::Error;

#[derive(Debug)]
pub struct WinState {
    pub(crate) id: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

#[derive(Debug)]
pub struct WMState<'a> {
    pub(crate) conn: &'a RustConnection,
    pub(crate) config: Config,
    screen_num: usize,
    pub(crate) running: bool,
    windows: Vec<WinState>,
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

impl<'a> WMState<'a> {
    pub fn new(conn: &'a RustConnection, screen_num: usize, config: Config) -> Self {
        Self {
            conn,
            config,
            screen_num,
            running: true,
            windows: vec![],
            selected_window: None,
        }
    }

    pub(crate) fn find_window_by_id(&self, id: Window) -> Option<&WinState> {
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

    fn manage_window(&mut self, window: Window) -> Result<(), ReplyOrIdError> {
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
            (EventMask::ButtonPress | EventMask::ButtonRelease | EventMask::PointerMotion) as u16;
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

    /// Handle events from the X server
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

    /// Handle a client from the socket
    pub fn handle_client(&mut self, stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
        // First for bytes we read should be the length of the command that follows
        let mut cmd_len = [0; 4];
        stream.read_exact(&mut cmd_len)?;
        // If it can't be parsed to a number we simply don't care about it
        let cmd_len = String::from_utf8(cmd_len.to_vec())?.parse::<usize>()?;

        let mut handle = stream.take(cmd_len as u64);
        let mut cmd = String::with_capacity(cmd_len);
        handle.read_to_string(&mut cmd)?;

        // TODO parse and handle the command

        Ok(())
    }
}
