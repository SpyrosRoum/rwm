mod command_handlers;
mod event_handlers;

use std::{error::Error, io::Read, os::unix::net::UnixStream, str::FromStr};

use x11rb::{
    connection::Connection,
    errors::ReplyOrIdError,
    protocol::{xproto::*, Event},
    rust_connection::RustConnection,
};

use crate::{
    command::{Command, LayoutSubcommand},
    config::Config,
    focus_history::FocusHist,
    layouts::LayoutType,
    states::{TagState, WinState},
};

#[derive(Debug)]
pub struct WMState<'a> {
    pub(crate) conn: &'a RustConnection,
    pub(crate) config: Config,
    screen_num: usize,
    pub(crate) running: bool,
    /// A vecDequeue with all windows that acts as a focus history as well
    pub(crate) windows: FocusHist,
    /// If this is Some, we are currently dragging the given window with the given offset relative
    /// to the mouse.
    pub(crate) selected_window: Option<(Window, (i16, i16))>,
    /// The tags that are currently visible
    pub(crate) tags: Vec<TagState>,
    pub(crate) layout: LayoutType,
}

impl<'a> WMState<'a> {
    pub(crate) fn new(conn: &'a RustConnection, screen_num: usize, config: Config) -> Self {
        let def_layout = config.layouts[0];
        // tags are 1-9 and the default is 1
        let mut tags: Vec<TagState> = (1..=9)
            .map(|i| TagState::new(i, false, def_layout).unwrap())
            .collect();
        tags[0].visible = true;
        Self {
            conn,
            config,
            screen_num,
            running: true,
            windows: FocusHist::new(),
            selected_window: None,
            tags,
            layout: def_layout,
        }
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

        // Note: We don't call self.update_windows() or self.layout.update()
        // because both get called in self.manage_window()
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
        // We also push at the front of the focus history because the window now has focus
        self.windows
            .push_front(WinState::new(window, &geom, self.tags.as_slice()));

        self.update_windows()?;
        Ok(())
    }

    /// Called when a window gets destroyed (DestroyNotify)
    fn unmanage_window(&mut self, window: Window) -> Result<(), ReplyOrIdError> {
        self.conn.unmap_window(window)?;
        self.conn
            .ungrab_button(ButtonIndex::Any, window, ModMask::Any)?;

        self.windows.forget(window, self.tags.as_slice());
        self.update_windows()
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
            Command::Layout(sub) => {
                self.layout = match sub {
                    LayoutSubcommand::Next => self.layout.next(&self.config.layouts),
                    LayoutSubcommand::Prev => self.layout.prev(&self.config.layouts),
                };
                self.update_windows()?
            }
        }

        Ok(())
    }

    /// Update the currently visible windows
    pub(crate) fn update_windows(&mut self) -> Result<(), ReplyOrIdError> {
        // Should this be replaced entirely by layout.update()?

        // Map the proper windows and unmap the rest
        for win in self.windows.iter() {
            if self
                .tags
                .iter()
                .any(|tag_state| tag_state.visible && win.tags.contains(&tag_state.id))
            {
                let attrs = ChangeWindowAttributesAux::default()
                    .border_pixel(self.config.normal_border_color);
                self.conn.change_window_attributes(win.id, &attrs)?;
                // TODO I can probably do this better than just setting the border width for every window again
                // This is done so if the master win gets dragged, it has a border
                self.conn.configure_window(
                    win.id,
                    &ConfigureWindowAux::new().border_width(self.config.border_width),
                )?;
                self.conn.map_window(win.id)?;
            } else {
                self.conn.unmap_window(win.id)?;
            }
        }

        if let Some(focused) = self.windows.get_focused() {
            let attrs =
                ChangeWindowAttributesAux::default().border_pixel(self.config.focused_border_color);
            self.conn.change_window_attributes(focused.id, &attrs)?;
        }

        let visible_tags = self
            .tags
            .iter()
            .filter(|tag_state| tag_state.visible)
            .map(|tag_state| tag_state.id)
            .collect::<Vec<_>>();
        self.layout.update(
            &self.conn,
            &self.windows,
            visible_tags,
            self.screen_num,
            self.config.border_width,
        )
    }
}
