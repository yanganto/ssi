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
                .help("Change the log level, accept level: all, trace, debug, info, warn, error[default]"),
        )
        .arg(
            Arg::with_name("exactly")
                .short("e")
                .long("exactly")
                .help("Get the value from the exactly node and not including children"),
        )
        .arg(
            Arg::with_name("all node")
                .short("a")
                .long("all-node")
                .help("Return the value in all type of node not only in the leaf node"),
        )
        .arg(
            Arg::with_name("root hash")
                .short("r")
                .long("root-hash")
                .takes_value(true)
                .help("The hash for trie root node, ex: 0x3b559d574c4a9f13e55d0256655f0f71a70a703766226f1080f80022e39c057d"),
        )
        .arg(
            Arg::with_name("storage key")
                .short("k")
                .long("storage-key")
                .takes_value(true)
                .help("The storage key you want to inspect, it is okay to use only prefix part of storage key, ex: 6aa394eea5630e07c48ae0c9558cef7"),
        )
        .arg(
            Arg::with_name("db path")
                .help("the db path to Rocks DB")
                .index(1)
                .requires("root hash")
                .requires("storage key")
                .required(true),
        )
        .get_matches_from(itr)
}
