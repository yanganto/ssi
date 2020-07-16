//! Error
use failure_derive::*;
use hex::FromHexError;

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
