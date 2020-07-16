// Comand Line Handle

use clap::{App, Arg, ArgMatches};
use std::ffi::OsString;

// use crate::character_map::SYMBOL_MAP;
// use crate::errors::*;
// use crate::extractor::SentenceExtractorBuilder;
// use crate::loader::load;
// use crate::loader::load_file_names;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn parse_args<'a, I, T>(itr: I) -> ArgMatches<'a>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    App::new("ssi")
        .about("Substrate Storage Inspector")
        .version(VERSION)
        .arg(
            Arg::with_name("log")
                .short("l")
                .long("log")
                .takes_value(true)
                .number_of_values(1)
                .help("change the log level, accept level: all, trace, debug, info, warn, error"),
        )
        .arg(
            Arg::with_name("exactly")
                .short("e")
                .long("exactly")
                .help("get the value from the exactly node and not including children"),
        )
        .arg(
            Arg::with_name("all node")
                .short("a")
                .long("all-node")
                .help("return the value in all type of node not only in the leaf node"),
        )
        .get_matches_from(itr)
}
