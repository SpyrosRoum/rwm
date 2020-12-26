use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::Path,
};

use anyhow::{bail, ensure, Context, Result};
use structopt::StructOpt;

use common::{into_message, Command};

fn main() -> Result<()> {
    let opts = Command::from_args();
    let message = into_message(opts).unwrap();

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

    let mut cmd_len = [0; 4];
    stream
        .read_exact(&mut cmd_len)
        .context("There was an error in the wm. Failed to read response.")?;
    let cmd_len = String::from_utf8(cmd_len.to_vec())
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let mut handle = stream.take(cmd_len as u64);
    let mut reply = String::with_capacity(cmd_len);
    handle
        .read_to_string(&mut reply)
        .context("Failed to read response.")?;
    let reply: String = serde_json::from_str(&reply).context("Wrong response format")?;
    println!("{}", reply);

    Ok(())
}
