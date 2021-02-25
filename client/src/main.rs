use std::{io::Write, os::unix::net::UnixStream, path::Path};

use {
    anyhow::{bail, ensure, Context, Result},
    structopt::StructOpt,
};

use common::{into_message, read_message, Command};

fn main() -> Result<()> {
    let opts = Command::from_args();
    let message = into_message(opts)?;

    let socket = Path::new("/tmp/rwm.sock");
    let mut stream = match UnixStream::connect(&socket) {
        Ok(s) => s,
        Err(_) => {
            bail!("Error connecting to the socket");
        }
    };

    ensure!(
        stream.write(message.as_ref()).is_ok(),
        "Error sending the command"
    );

    let reply = read_message(&mut stream)?;
    let reply: String = serde_json::from_str(&reply).context("Wrong response format")?;
    println!("{}", reply);

    Ok(())
}
