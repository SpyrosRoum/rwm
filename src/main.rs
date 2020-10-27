mod config;
mod states;

use std::process::exit;

use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::*;
use x11rb::protocol::ErrorKind;

use config::Config;
use states::WMState;

fn try_become_wm<C: Connection>(conn: &C, screen: &Screen) -> Result<(), ReplyError> {
    let change = ChangeWindowAttributesAux::default().event_mask(
        EventMask::SubstructureRedirect | EventMask::SubstructureNotify | EventMask::ButtonPress,
    );

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
    let mut wm_state = WMState::new(&conn, screen_num, Config::default());
    wm_state.scan_windows().unwrap();

    // Main loop
    loop {
        wm_state.conn.flush().unwrap();

        while let Some(event) = wm_state.conn.poll_for_event().unwrap() {
            wm_state.handle_event(event).unwrap();
        }
    }
}
