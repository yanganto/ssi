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

pub fn init_logger(logger: &'static Logger) {
    log::set_logger(logger).unwrap();
    log::set_max_level(LevelFilter::Trace);
}
