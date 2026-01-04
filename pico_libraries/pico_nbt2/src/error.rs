use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Invalid tag ID: {0}")]
    InvalidTagId(u8),

    #[error("Unexpected tag: expected {expected:?}, found {found:?}")]
    UnexpectedTag { expected: u8, found: u8 },

    #[error("Unexpected EOF")]
    Eof,

    #[error("Custom error: {0}")]
    Message(String),

    #[error("Trailing bytes")]
    TrailingBytes,
}

pub type Result<T> = std::result::Result<T, Error>;

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}
