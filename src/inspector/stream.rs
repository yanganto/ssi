use std::fs::File;
use std::io::{self, BufRead, Read};

use colored::*;
use regex::Regex;

use crate::cli::ArgMatches;
use crate::codec::storage_key_semantic_decode;
use crate::errors::Error;
use crate::logger::debug;

static HEXAL_LITERAL_FORMAT: &str = r#"[0-9a-f]{32,}"#;

struct Input<'a> {
    source: Box<dyn BufRead + 'a>,
}

impl<'a> Input<'a> {
    fn stdin(stdin: &'a io::Stdin) -> Input<'a> {
        Input {
            source: Box::new(stdin.lock()),
        }
    }

    fn file(path: &str) -> io::Result<Input<'a>> {
        File::open(path).map(|file| Input {
            source: Box::new(io::BufReader::new(file)),
        })
    }
}

impl<'a> Read for Input<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.source.read(buf)
    }
}

impl<'a> BufRead for Input<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.source.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.source.consume(amt);
    }
}

pub fn stream_inspect_app(matches: ArgMatches) -> Result<(), Error> {
    let stdin = io::stdin();
    let hex_re = Regex::new(HEXAL_LITERAL_FORMAT).unwrap();

    let input_stream = if let Some(path) = matches.value_of("path") {
        Input::file(path)?
    } else {
        Input::stdin(&stdin)
    };

    for line in input_stream.lines() {
        if let Ok(l) = line {
            println!("{}", l);
            let decode_value = hex_re
                .captures_iter(&l)
                .map(|cap| {
                    let semantic_result = storage_key_semantic_decode(&cap[0], true);
                    debug!("capture hex literal: {}", &cap[0]);
                    if semantic_result.0.is_some() {
                        format!(
                            "{} > {} > {}",
                            semantic_result.0.unwrap_or_default(),
                            semantic_result.1.unwrap_or_default(),
                            semantic_result.2.unwrap_or_default()
                        )
                    } else {
                        String::new()
                    }
                })
                .collect::<String>();
            if !decode_value.is_empty() {
                println!("{} {}", "==>".blue(), decode_value.blue());
            }
        }
    }
    Ok(())
}
