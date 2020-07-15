/// Logger
/// Customized logger
use colored::*;
pub use log::{debug, error, info, trace, Level};
use log::{LevelFilter, Metadata, Record};

pub struct Logger;
impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                log::Level::Error => println!("{} - {}", "ERROR".red().bold(), record.args()),
                log::Level::Warn => println!("{} - {}", "WARN".red(), record.args()),
                log::Level::Info => println!("{} - {}", "INFO".cyan(), record.args()),
                log::Level::Debug => println!("{} - {}", "DEBUG".blue().bold(), record.args()),
                log::Level::Trace => println!("{} - {}", "TRACE".blue(), record.args()),
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
