mod color;
mod config;
mod focus_history;
mod layouts;
mod mod_mask;
mod states;
mod utils;

use std::{io::Write, net::Shutdown, os::unix::net::UnixListener, path::PathBuf};

use anyhow::{Context, bail};
use structopt::StructOpt;
use x11rb::{
    connection::Connection,
    errors::ReplyError,
    protocol::{xproto::*, ErrorKind, Event},
    rust_connection::RustConnection,
};

use common::into_message;
use config::Config;
use states::WMState;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Optional path to a config file
    config: Option<PathBuf>,
    /// Prints the default configuration in stdout and exits
    #[structopt(short, long)]
    print: bool,
}

fn try_become_wm(conn: &RustConnection, screen: &Screen) -> Result<(), ReplyError> {
    let change = ChangeWindowAttributesAux::default().event_mask(
        EventMask::SubstructureRedirect
            | EventMask::SubstructureNotify
            | EventMask::ButtonPress
            | EventMask::StructureNotify
            | EventMask::PropertyChange,
    );

    conn.change_window_attributes(screen.root, &change)?.check()
}

fn main() -> anyhow::Result<()> {
    let options: Opt = Opt::from_args();
    if options.print {
        let config = Config::default();
        println!("{}", toml::to_string(&config)?);
        return Ok(());
    }
    let (conn, screen_num) = RustConnection::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];

    if let Err(err) = try_become_wm(&conn, screen) {
        if let ReplyError::X11Error(error) = err {
            if error.error_kind == ErrorKind::Access {
                bail!("Another WM in already running.");
            } else {
                bail!("Something went wrong");
            }
        }
    };

    // We are the window manager!
    let mut config = Config::default();
    if let Some(path) = options.config {
        config
            .load(Some(path.clone()))
            .with_context(|| format!("Failed to load configuration file {:?}", path))?;
    }
    let mut wm_state = WMState::new(&conn, screen_num, config);
    wm_state.scan_windows().context("Error while looking for pre-existing windows")?;

    let listener = UnixListener::bind("/tmp/rwm.sock").context("Failed to connect to socket")?;
    listener.set_nonblocking(true).unwrap();

    let poller = polling::Poller::new().unwrap();
    poller
        .add(conn.stream(), polling::Event::readable(1))
        .unwrap();
    poller.add(&listener, polling::Event::readable(2)).unwrap();
    // events from poller go here
    let mut events = Vec::new();

    let mut last_motion = 0;
    // Main loop
    while wm_state.running {
        wm_state.conn.flush().unwrap();
        poller.wait(&mut events, None).unwrap();
        // We just want to iterate and modify them so we wait for the next event as well
        // By default once it gets the first event from a source it doesn't wait for another one again..
        // We use drain() because we want to clear the event for the next to go in
        events.drain(..).for_each(|ev| {
            if ev.key == 1 {
                poller
                    .modify(conn.stream(), polling::Event::readable(1))
                    .unwrap();
            } else if ev.key == 2 {
                poller
                    .modify(&listener, polling::Event::readable(2))
                    .unwrap();
            }
        });

        while let Some(event) = wm_state.conn.poll_for_event().unwrap() {
            if let Event::MotionNotify(ev) = &event {
                // This is done so we don't update the window for every pixel we move/resize it
                if ev.time - last_motion < 1000 / 144 {
                    continue;
                }
                last_motion = ev.time;
            }
            // ToDo Error handling
            wm_state.handle_event(event).unwrap();
        }

        if let Ok((mut stream, _adr)) = listener.accept() {
            let reply = match wm_state.handle_client(&mut stream) {
                Ok(msg) => into_message(msg).unwrap(),
                Err(e) => into_message(format!("{:?}", e)).unwrap(),
            };
            stream.write(reply.as_ref()).ok(); // .ok() is used to basically just ignore the result
            stream.shutdown(Shutdown::Both).ok();
        };
    }

    utils::clean_up().context("Failed to clean up.")?;
    Ok(())
}
