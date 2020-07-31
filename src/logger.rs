/// Logger
/// Customized logger
///
/// The log is classified into following level
/// - `error` log the unexpect behavior cause by user input
/// - `warn` log the unexpect db struct
/// - `info` log the information about the trie node
/// - `debug` log the tracing trie operation
/// - `trace` log the get/contain operations in DB
///
use colored::*;
pub use log::{debug, error, info, trace, warn, Level};
use log::{LevelFilter, Metadata, Record};

pub struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                log::Level::Error => eprintln!("{} - {}", "ERROR".red().bold(), record.args()),
                log::Level::Warn => eprintln!("{} - {}", "WARN".red(), record.args()),
                log::Level::Info => eprintln!("{} - {}", "INFO".cyan(), record.args()),
                log::Level::Debug => eprintln!("{} - {}", "DEBUG".blue().bold(), record.args()),
                log::Level::Trace => eprintln!("{} - {}", "TRACE".blue(), record.args()),
            }
        }
    }
    fn flush(&self) {}
}

pub fn init_logger(logger: &'static Logger, log_level: &str) {
    log::set_logger(logger).unwrap();
    if log_level == "trace" || log_level == "all" {
        log::set_max_level(LevelFilter::Trace);
    } else if log_level == "debug" {
        log::set_max_level(LevelFilter::Debug);
    } else if log_level == "info" {
        log::set_max_level(LevelFilter::Info);
    } else if log_level == "warn" {
        log::set_max_level(LevelFilter::Warn);
    } else {
        log::set_max_level(LevelFilter::Error);
    }
}
