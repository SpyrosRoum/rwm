mod config;
mod event_handlers;
mod states;
mod utils;

use std::process::exit;
use std::os::unix::{io::AsRawFd, net::UnixListener};

use polling;
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::*;
use x11rb::protocol::ErrorKind;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use config::Config;
use states::WMState;

fn try_become_wm(conn: &RustConnection, screen: &Screen) -> Result<(), ReplyError> {
    let change = ChangeWindowAttributesAux::default().event_mask(
        EventMask::SubstructureRedirect
            | EventMask::SubstructureNotify
            | EventMask::ButtonPress
            | EventMask::StructureNotify,
    );

    conn.change_window_attributes(screen.root, &change)?.check()
}

fn main() {
    let (conn, screen_num) = RustConnection::connect(None).unwrap();
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

    let listener = UnixListener::bind("/tmp/rwm.sock").expect("Failed connect to socket");
    listener.set_nonblocking(true).unwrap();

    let poller = polling::Poller::new().unwrap();
    poller.add(conn.stream(), polling::Event::readable(1));
    poller.add(&listener, polling::Event::readable(2));
    // events from poller go here
    let mut events = Vec::new();

    let mut last_motion = 0;
    // Main loop
    loop {
        wm_state.conn.flush().unwrap();
        poller.wait(&mut events, None);
        // We just want to iterate and modify them so we wait for the next event as well
        // By default once it gets the first event from a source it doesn't wait for another one again..
        events
            .iter()
            .map(|ev| {
                if ev.key == 1 {
                    poller
                        .modify(conn.stream(), polling::Event::readable(1))
                        .unwrap();
                } else if ev.key == 2 {
                    poller
                        .modify(&listener, polling::Event::readable(2))
                        .unwrap();
                }
            })
            .collect::<Vec<_>>(); // We need to collect it because iterators are lazy and do nothing if not used

        while let Some(event) = wm_state.conn.poll_for_event().unwrap() {
            if let Event::MotionNotify(ev) = &event {
                // This is done so we don't update the window for every pixel we move/resize it
                if ev.time - last_motion < 1000 / 144 {
                    continue;
                }
                last_motion = ev.time;
            }
            wm_state.handle_event(event).unwrap();
        }

        if let Ok(stream) = listener.accept() {
            // TODO handle the stream
            dbg!(&stream);
        };
        // Clean the poller events so new can go in
        events.clear();
    }
}
