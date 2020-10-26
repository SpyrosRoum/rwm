use std::process::exit;

use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::{ErrorKind, Event};

const BORDER_WIDTH: u32 = 4;
const BORDER_COLOR: u32 = 0b11111111_00000000_00000000_11111111; // ARGB format

#[derive(Debug)]
struct WinState {
    id: Window,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
}

impl WinState {
    fn new(win: Window, geom: &GetGeometryReply) -> Self {
        Self {
            id: win,
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: geom.height,
        }
    }
}

#[derive(Debug)]
struct WMState<'a, C: Connection> {
    conn: &'a C,
    screen_num: usize,
    windows: Vec<WinState>,
}

impl<'a, C> WMState<'a, C>
where
    C: Connection,
{
    fn new(conn: &'a C, screen_num: usize) -> Self {
        Self {
            conn,
            screen_num,
            windows: vec![],
        }
    }

    /// Scan for pre-existing windows and manage them
    fn scan_windows(&mut self) -> Result<(), ReplyOrIdError> {
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
        let config = ConfigureWindowAux::default().border_width(BORDER_WIDTH);
        self.conn.configure_window(window, &config)?;

        // Give color to the border
        let attrs = ChangeWindowAttributesAux::default().border_pixel(BORDER_COLOR);
        self.conn.change_window_attributes(window, &attrs)?;

        // Show the window
        self.conn.map_window(window)?;

        let geom = self.conn.get_geometry(window)?.reply()?;
        self.windows.push(WinState::new(window, &geom));

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<(), ReplyOrIdError> {
        match event {
            Event::MapRequest(event) => self.manage_window(event.window)?,
            _ => {}
        }

        Ok(())
    }
}

fn try_become_wm<C: Connection>(conn: &C, screen: &Screen) -> Result<(), ReplyError> {
    let change = ChangeWindowAttributesAux::default()
        .event_mask(EventMask::SubstructureRedirect | EventMask::StructureNotify);

    conn.change_window_attributes(screen.root, &change)?.check()
}

fn main() {
    let (conn, screen_num) = x11rb::connect(None).unwrap();

    let screen = &conn.setup().roots[screen_num];

    if let Err(err) = try_become_wm(&conn, screen) {
        if let ReplyError::X11Error(error) = err {
            if error.error_kind == ErrorKind::Access {
                eprintln!("Another WM in already running.");
                exit(1);
            } else {
                eprintln!("Error");
                exit(1);
            }
        }
    };

    // We are the window manager!
    let mut wm_state = WMState::new(&conn, screen_num);
    wm_state.scan_windows().unwrap();

    // Main loop
    loop {
        wm_state.conn.flush().unwrap();

        while let Some(event) = wm_state.conn.poll_for_event().unwrap() {
            wm_state.handle_event(event).unwrap();
        }
    }
}
