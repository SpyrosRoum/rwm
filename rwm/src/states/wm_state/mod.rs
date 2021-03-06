mod command_handlers;
mod event_handlers;

use std::{collections::HashSet, os::unix::net::UnixStream};

use {
    anyhow::Context,
    x11rb::{
        connection::Connection,
        cursor::Handle as CursorHandle,
        errors::ReplyOrIdError,
        protocol::{xproto::*, Event},
        rust_connection::RustConnection,
    },
};

use crate::{
    config::Config,
    focus_history::FocusHist,
    layouts::LayoutType,
    states::{TagState, WinState},
    utils,
};
use common::{Command, ConfigSubcommand, LayoutSubcommand};

#[derive(Debug)]
pub(crate) struct WmState<'a> {
    pub(crate) conn: &'a RustConnection,
    pub(crate) config: Config,
    screen_num: usize,
    pub(crate) running: bool,
    /// A vecDequeue with all windows that acts as a focus history as well
    pub(crate) windows: FocusHist,
    /// If this is Some, we are currently dragging the given window with the given offset relative
    /// to the mouse.
    pub(crate) dragging_window: Option<(Window, (i16, i16))>,
    /// Same as `dragging_window` but for resizing.
    pub(crate) resizing_window: Option<(Window, (i16, i16))>,
    /// The tags that are currently visible
    pub(crate) tags: Vec<TagState>,
    pub(crate) layout: LayoutType,
    pub(crate) cursor_handle: CursorHandle,
}

impl<'a> WmState<'a> {
    pub(crate) fn new(
        conn: &'a RustConnection,
        screen_num: usize,
        config: Config,
        cursor_handle: CursorHandle,
    ) -> Self {
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
            dragging_window: None,
            resizing_window: None,
            tags,
            layout: def_layout,
            cursor_handle,
        }
    }

    /// Apply user defined rules on the given window (ex put it in tag 2 by default)
    fn apply_rules(&mut self, window: &mut WinState) -> Result<(), ReplyOrIdError> {
        if self.config.class_rules.is_empty() && self.config.name_rules.is_empty() {
            // No rules to apply
            return Ok(());
        }

        let class_names = self
            .conn
            .get_property(
                false,
                window.id,
                AtomEnum::WM_CLASS,
                AtomEnum::STRING,
                0,
                1024,
            )?
            .reply()?
            .value; // Alternative we would do .value8().unwrap().collect::<Vec<_>>();

        // We should usually get two values, technically the first is the instance name and the second one the class name
        // Technically we should check against the second value only but instead we will check against both
        // I believe some apps only return one value (could be wrong on that) but also checking both is more user friendly
        // If there is some reason to *not* do this please raise an issue!
        let class_names = String::from_utf8(class_names)
            .expect("utf 8")
            .trim_matches('\0')
            .split('\0')
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        for name in class_names.iter() {
            if let Some(tag_ids) = self.config.class_rules.get(name) {
                let mut tags = HashSet::new();
                for tag_id in tag_ids.iter() {
                    tags.insert(tag_id.to_owned());
                }
                window.tags = tags;

                return Ok(());
            }
        }

        let wm_name = self
            .conn
            .get_property(
                false,
                window.id,
                AtomEnum::WM_NAME,
                AtomEnum::STRING,
                0,
                1024,
            )?
            .reply()?
            .value;
        let wm_name = String::from_utf8(wm_name).expect("utf 8");

        if let Some(tag_ids) = self.config.class_rules.get(&wm_name) {
            let mut tags = HashSet::new();
            for tag_id in tag_ids.iter() {
                tags.insert(tag_id.to_owned());
            }
            window.tags = tags;
        }

        Ok(())
    }

    /// Scan for pre-existing windows and manage them
    pub(crate) fn scan_windows(&mut self) -> Result<(), ReplyOrIdError> {
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
            if !attr.override_redirect && attr.map_state != MapState::UNMAPPED {
                self.manage_window(win)?;
            }
        }
        if let Some(new_focused) = self.windows.find_next(&self.tags).map(|(_, new)| new) {
            let id = new_focused.id;
            self.focus(id)?;
        };

        self.update_windows()
    }

    fn manage_window(&mut self, window: Window) -> Result<(), ReplyOrIdError> {
        // Add a border
        let config = ConfigureWindowAux::default().border_width(self.config.border_width);
        self.conn.configure_window(window, &config)?;

        // Register the proper events with the window
        let events = ChangeWindowAttributesAux::default().event_mask(
            EventMask::ENTER_WINDOW
                | EventMask::FOCUS_CHANGE
                | EventMask::PROPERTY_CHANGE
                | EventMask::STRUCTURE_NOTIFY,
        );
        self.conn
            .change_window_attributes(window, &events)?
            .check()?;

        // Get Button Press events
        utils::grab_buttons(&self.conn, window, self.config.mod_key, false)?;

        // Show the window
        self.conn.map_window(window)?;

        let geom = self.conn.get_geometry(window)?.reply()?;

        // We give a reference to the tags so the window can deduce what tags are currently visible.
        // We also push at the front of the focus history because the window now has focus
        let mut window = WinState::new(window, &geom, self.tags.as_slice());

        // If it's a transient window then copy parent's tags and make it floating
        if let Ok(Some(id)) = utils::get_transient_for(&self.conn, window.id) {
            if let Some((_, parent_window)) = self.windows.find_by_id(id) {
                let _ = std::mem::replace(&mut window.tags, parent_window.tags.clone());
                window.floating = true;
            }
        }

        // Apply the user defined rules about where the window should spawn
        self.apply_rules(&mut window)?;

        self.windows.push_front(window);
        Ok(())
    }

    /// Called when a window gets destroyed (DestroyNotify)
    fn unmanage_window(&mut self, window: Window) -> Result<(), ReplyOrIdError> {
        self.conn.unmap_window(window)?;

        if let Some(new_focused) = self.windows.forget(window, self.tags.as_slice()) {
            let id = new_focused.id;
            self.focus(id)?;
        }

        self.conn
            .ungrab_button(ButtonIndex::ANY, window, ModMask::ANY)?;

        self.update_windows()
    }

    /// Handle events from the X server
    pub(crate) fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        match event {
            Event::MapRequest(event) => {
                self.manage_window(event.window)?;
                self.focus(event.window)?;
                self.update_windows()?;
            }
            Event::ButtonPress(event) => self.on_button_press(event)?,
            Event::ButtonRelease(event) => self.on_button_release(event)?,
            Event::MotionNotify(event) => self.on_motion_notify(event)?,
            Event::DestroyNotify(event) => self.unmanage_window(event.window)?,
            Event::EnterNotify(event) => self.on_enter_notify(event)?,
            Event::PropertyNotify(event) => self.on_property_notify(event)?,
            _ => {}
        };
        Ok(())
    }

    /// Handle a client from the socket
    pub(crate) fn handle_client(&mut self, stream: &mut UnixStream) -> anyhow::Result<String> {
        let cmd = common::read_message(stream)?;
        let cmd: Command = serde_json::from_str(&cmd).context("Invalid command")?;

        self.handle_command(cmd)
    }

    /// Handle the command from a client
    fn handle_command(&mut self, cmd: Command) -> anyhow::Result<String> {
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
                self.update_windows().with_context(|| {
                    format!("Failed to update windows after `Layout({:?})`", sub)
                })?
            }
            Command::Config(ConfigSubcommand::Print) => {
                return ron::ser::to_string_pretty(&self.config, ron::ser::PrettyConfig::default())
                    .context("Failed to serialise current configuration");
            }
            Command::Config(ConfigSubcommand::Load { path }) => {
                self.config.load(path)?;
                self.update_windows()
                    .context("Failed to update windows after loading configuration")?;
            }
        }

        Ok(String::from("0"))
    }

    /// Update the currently visible windows
    pub(crate) fn update_windows(&mut self) -> Result<(), ReplyOrIdError> {
        // Should this be replaced entirely by layout.update()?

        // Map the proper windows and unmap the rest
        for win in self.windows.iter() {
            if utils::is_visible(win, &self.tags) {
                self.conn.map_window(win.id)?;
            } else {
                self.conn.unmap_window(win.id)?;
            }
        }

        if self.windows.get_focused().is_none() {
            // Give input focus to root window, otherwise no input is possible
            let root = self.conn.setup().roots[self.screen_num].root;
            self.conn
                .set_input_focus(InputFocus::NONE, root, x11rb::CURRENT_TIME)?;
        }

        let visible_tags = self
            .tags
            .iter()
            .filter(|tag_state| tag_state.visible)
            .map(|tag_state| tag_state.id)
            .collect::<Vec<_>>();
        self.layout.update(
            &self.conn,
            &mut self.windows,
            visible_tags,
            self.screen_num,
            self.config.border_width,
            self.config.gap,
        )
    }

    /// A helper method for setting the cursor
    pub(crate) fn set_cursor(
        &self,
        window: Window,
        cursor_name: &str,
    ) -> Result<(), ReplyOrIdError> {
        self.conn.change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new()
                .cursor(self.cursor_handle.load_cursor(self.conn, cursor_name)?),
        )?;

        Ok(())
    }

    pub(crate) fn focus(&mut self, id: Window) -> Result<(), ReplyOrIdError> {
        if let Some(old_focused) = self.windows.get_focused() {
            if old_focused.id == id {
                return Ok(());
            }
            utils::grab_buttons(&self.conn, old_focused.id, self.config.mod_key, false)?;

            let attrs =
                ChangeWindowAttributesAux::default().border_pixel(self.config.normal_border_color);
            self.conn.change_window_attributes(old_focused.id, &attrs)?;
        }

        utils::grab_buttons(&self.conn, id, self.config.mod_key, true)?;
        self.windows.set_focused(id);

        // Give it the correct border color
        let attrs =
            ChangeWindowAttributesAux::default().border_pixel(self.config.focused_border_color);
        self.conn.change_window_attributes(id, &attrs)?;
        // Give keyboard input to window
        self.conn
            .set_input_focus(InputFocus::NONE, id, x11rb::CURRENT_TIME)?;

        Ok(())
    }
}
