mod color;
mod config;
mod focus_history;
mod layouts;
mod mod_mask;
mod spawn_rule;
mod states;
mod utils;

use std::{io::Write, net::Shutdown, os::unix::net::UnixListener, path::PathBuf};

use {
    anyhow::{bail, Context},
    structopt::StructOpt,
    x11rb::{
        connection::Connection,
        errors::ReplyError,
        protocol::{xproto::*, ErrorKind, Event},
        rust_connection::RustConnection,
    },
};

use common::into_message;
use {config::Config, states::WmState};

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
        EventMask::SUBSTRUCTURE_REDIRECT
            | EventMask::SUBSTRUCTURE_NOTIFY
            | EventMask::BUTTON_PRESS
            | EventMask::STRUCTURE_NOTIFY
            | EventMask::PROPERTY_CHANGE,
    );
    conn.change_window_attributes(screen.root, &change)?.check()
}

fn main() -> anyhow::Result<()> {
    let options: Opt = Opt::from_args();
    if options.print {
        let config = Config::default();
        println!(
            "{}",
            ron::ser::to_string_pretty(&config, ron::ser::PrettyConfig::default())?
        );
        return Ok(());
    }

    let (conn, screen_num) =
        RustConnection::connect(None).context("Failed to connect to the X server")?;
    let screen = &conn.setup().roots[screen_num];

    if let Err(ReplyError::X11Error(error)) = try_become_wm(&conn, screen) {
        if error.error_kind == ErrorKind::Access {
            bail!("Another WM in already running.");
        } else {
            bail!("Something went wrong");
        }
    };

    // We are the window manager!

    let config = match options.config {
        Some(path) => Config::from_file(path)?,
        None => Config::default(),
    };

    if config.layouts.is_empty() {
        bail!("There needs to be at least one layout in the config");
    }

    let mut wm_state = WmState::new(&conn, screen_num, config);
    wm_state
        .scan_windows()
        .context("Error while looking for pre-existing windows")?;

    let listener = UnixListener::bind("/tmp/rwm.sock").context("Failed to connect to socket")?;
    listener.set_nonblocking(true).unwrap();

    let poller = polling::Poller::new().unwrap();
    poller
        .add(conn.stream(), polling::Event::readable(1))
        .context("epoll add failed")?;
    poller
        .add(&listener, polling::Event::readable(2))
        .context("epoll add failed")?;
    // events from poller go here
    let mut events = Vec::new();

    let mut last_motion = 0;
    // Main loop
    while wm_state.running {
        wm_state.conn.flush().context("Error talking to X server")?;
        if poller.wait(&mut events, None).is_err() {
            // ToDo It's possible I should handle and exit on some errors
            continue;
        }
        // We just want to iterate and modify them so we wait for the next event as well
        // By default once it gets the first event from a source it doesn't wait for another one again..
        // We use drain() because we want to clear the event for the next to go in
        for ev in events.drain(..) {
            if ev.key == 1 {
                poller
                    .modify(conn.stream(), polling::Event::readable(1))
                    .context("Error setting the interest for new events")?;
            } else if ev.key == 2 {
                poller
                    .modify(&listener, polling::Event::readable(2))
                    .context("Error setting the interest for new events")?;
            }
        }

        while let Some(event) = wm_state
            .conn
            .poll_for_event()
            .context("Error talking to X server")?
        {
            if let Event::MotionNotify(ev) = &event {
                // This is done so we don't update the window for every pixel we move/resize it
                if ev.time - last_motion < 1000 / 144 {
                    continue;
                }
                last_motion = ev.time;
            }
            // ToDo Error handling
            wm_state.handle_event(event)?;
        }

        if let Ok((mut stream, _adr)) = listener.accept() {
            let reply = match wm_state.handle_client(&mut stream) {
                Ok(msg) => into_message(msg),
                Err(e) => into_message(format!("{:?}", e)),
            }
            .unwrap_or_else(|_| into_message("Failed to serialise original message").unwrap());
            stream.write(reply.as_ref()).ok(); // .ok() is used to basically just ignore the result
            stream.shutdown(Shutdown::Both).ok();
        };
    }

    utils::clean_up().context("Failed to clean up.")
}
