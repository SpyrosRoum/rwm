use std::process::exit;

use x11rb;
use x11rb::connection::Connection;
use x11rb::errors::{ReplyError, ReplyOrIdError};
use x11rb::protocol::xproto::*;
use x11rb::protocol::ErrorKind;

struct WinState {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct WMState<C: Connection> {
    connection: C,
    screen_num: usize,
    focus_hist: Vec<WinState>,
}

impl<C> WMState<C>
where
    C: Connection,
{
    fn new(conn: C, screen_num: usize) -> Self {
        Self {
            connection: conn,
            screen_num,
            focus_hist: vec![],
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
    let wm_state = WMState::new(conn, screen_num);
}
