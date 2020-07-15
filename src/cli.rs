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
        .get_matches_from(itr)
}
