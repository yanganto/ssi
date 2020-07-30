use std::env::args_os;

mod logger;
use logger::{init_logger, Logger};

mod storage;

mod cli;
use cli::parse_args;

mod errors;

mod codec;

mod inspector;
use inspector::{db_inspect_app, decode_storage_key, stream_inspect_app};

static LOGGER: Logger = Logger;

fn main() {
    let matches = parse_args(args_os());
    init_logger(&LOGGER, matches.value_of("log").unwrap_or("error"));

    let f = if matches.is_present("decode storage key") {
        if matches.is_present("storage key") {
            decode_storage_key
        } else {
            stream_inspect_app
        }
    } else if matches.is_present("path") {
        db_inspect_app
    } else {
        println!("uncorrect usage: -h or --help to learn more");
        return;
    };

    if let Err(e) = f(matches) {
        println!("{}", e);
    }
}
