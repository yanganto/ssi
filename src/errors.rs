//! Error
use failure_derive::*;
use hex::FromHexError;
use std::io::Error as IOError;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Options {} is not correct, due to {}", 0, 1)]
    OptionValueIncorrect(String, String),
}

impl From<FromHexError> for Error {
    fn from(e: FromHexError) -> Self {
        Error::OptionValueIncorrect("state root hash".to_string(), format!("{}", e))
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::OptionValueIncorrect("path error".to_string(), format!("{}", e))
    }
}
