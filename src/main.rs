use std::process::exit;

use x11rb;
use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::ErrorKind;

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
}
