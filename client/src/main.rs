use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    path::Path,
    process::exit,
};

use common::{into_message, Command};
use structopt::StructOpt;

fn main() {
    let opts = Command::from_args();
    let message = into_message(opts).unwrap();

    let socket = Path::new("/tmp/rwm.sock");
    let mut stream = match UnixStream::connect(&socket) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error connecting to the socket");
            exit(1);
        }
    };

    if stream.write(message.as_ref()).is_err() {
        eprintln!("Error sending the command");
        exit(1);
    }

    let mut r = [0u8; 1];
    stream.read_exact(&mut r).ok();
    let r = match String::from_utf8(r.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Error getting reply from rwm.");
            exit(1);
        }
    };
    println!("Answer: {}", r);
}
