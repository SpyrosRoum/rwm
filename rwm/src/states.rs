use std::{
    collections::{HashSet, VecDeque},
    error::Error,
    io::Read,
    os::unix::net::UnixStream,
    str::FromStr,
};

use x11rb::{
    connection::Connection,
    errors::ReplyOrIdError,
    protocol::{xproto::*, Event},
    rust_connection::RustConnection,
};

use crate::{command::Command, config::Config, newtypes::Tag};

#[derive(Debug)]
pub struct WinState {
    pub(crate) id: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
    /// The tags that this window is on
    pub(crate) tags: HashSet<Tag>,
}

#[derive(Debug)]
pub struct WMState<'a> {
    pub(crate) conn: &'a RustConnection,
    pub(crate) config: Config,
    screen_num: usize,
    pub(crate) running: bool,
    /// A vecDequeue with all windows that acts as a focus history as well
    pub(crate) windows: VecDeque<WinState>,
    /// If this is Some, we are currently dragging the given window with the given offset relative
    /// to the mouse.
    pub(crate) selected_window: Option<(Window, (i16, i16))>,
    /// The tags that are currently visible
    pub(crate) tags: HashSet<Tag>,
}

impl WinState {
    pub(crate) fn new(win: Window, geom: &GetGeometryReply, tags: HashSet<Tag>) -> Self {
        Self {
            id: win,
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: geom.height,
            tags,
        }
    }
}

impl<'a> WMState<'a> {
    pub fn new(conn: &'a RustConnection, screen_num: usize, config: Config) -> Self {
        // tags are 1-9 and the default is 1
        let mut tags = HashSet::with_capacity(9);
        tags.insert(Tag::new(1).unwrap());
        Self {
            conn,
            config,
            screen_num,
            running: true,
            windows: VecDeque::new(),
            selected_window: None,
            tags,
        }
    }

    /// Get the Window State and the index of it in the vec
    pub(crate) fn find_window_by_id(&self, id: Window) -> Option<(usize, &WinState)> {
        self.windows
            .iter()
            .enumerate()
            .find(|(_i, win)| win.id == id)
    }

    pub(crate) fn get_focused_window(&self) -> Option<&WinState> {
        self.windows.iter().find(|win_state| {
            for tag in self.tags.iter() {
                return win_state.tags.contains(tag);
            }
            // There is always at least one tag
            unreachable!();
        })
    }

    pub(crate) fn get_focused_window_mut(&mut self) -> Option<&mut WinState> {
        // This can probably be done better without cloning
        let tags = self.tags.clone();
        self.windows.iter_mut().find(|win_state| {
            for tag in tags.iter() {
                return win_state.tags.contains(tag);
            }
            // There is always at least one tag
            unreachable!();
        })
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

        // Register the proper events with the window
        let events = ChangeWindowAttributesAux::default().event_mask(
            EventMask::KeyPress
                | EventMask::KeyRelease
                | EventMask::ButtonRelease
                | EventMask::PointerMotion
                | EventMask::EnterWindow
                | EventMask::FocusChange
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
        // self.tags.clone() because the new window will be in the currently viewable tags
        self.windows
            .push_back(WinState::new(window, &geom, self.tags.clone()));

        self.update_windows()?;
        Ok(())
    }

    /// Called when a window gets destroyed (DestroyNotify)
    fn unmanage_window(&mut self, window: Window) -> Result<(), ReplyOrIdError> {
        self.conn.unmap_window(window)?;
        self.conn
            .ungrab_button(ButtonIndex::Any, window, ModMask::Any)?;
        self.windows.retain(|win_state| win_state.id != window);
        Ok(())
    }

    /// Handle events from the X server
    pub fn handle_event(&mut self, event: Event) -> Result<(), ReplyOrIdError> {
        match event {
            Event::MapRequest(event) => self.manage_window(event.window)?,
            Event::ButtonPress(event) => self.on_button_press(event)?,
            Event::ButtonRelease(event) => self.on_button_release(event)?,
            Event::MotionNotify(event) => self.on_motion_notify(event)?,
            Event::DestroyNotify(event) => self.unmanage_window(event.window)?,
            Event::FocusIn(event) => self.on_focus_in(event)?,
            Event::EnterNotify(event) => self.on_enter_notify(event)?,
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
        let cmd = Command::from_str(&cmd)?;
        self.handle_command(cmd)?;

        Ok(())
    }

    /// Handle the command from a client
    fn handle_command(&mut self, cmd: Command) -> Result<(), Box<dyn Error>> {
        match cmd {
            Command::Quit => {
                self.running = false;
            }
            Command::Tag(sub) => self.on_tag_cmd(sub)?,
            Command::Window(sub) => self.on_window_cmd(sub)?,
        }

        Ok(())
    }

    /// Called when there is a change like a tag introduced
    pub(crate) fn update_windows(&mut self) -> Result<(), ReplyOrIdError> {
        let mut found_first = false;

        for win in self.windows.iter() {
            for tag in self.tags.iter() {
                if win.tags.contains(tag) {
                    let border_color = if !found_first {
                        found_first = true;
                        self.config.focused_border_color
                    } else {
                        self.config.normal_border_color
                    };
                    let attrs = ChangeWindowAttributesAux::default().border_pixel(border_color);
                    self.conn.change_window_attributes(win.id, &attrs)?;

                    self.conn.map_window(win.id)?;
                    break;
                } else {
                    self.conn.unmap_window(win.id)?;
                }
            }
        }

        Ok(())
    }
}
