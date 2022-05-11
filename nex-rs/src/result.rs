use crate::{nex_types::ResultCode, server};
use snafu::Snafu;
use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu(display(
        "Server error: {}",
        error.to_string()
    ))]
    ServerError { error: server::Error },
    #[snafu(display("Error: {}", message))]
    Generic { message: String },
}

impl From<server::Error> for Error {
    fn from(error: server::Error) -> Self {
        Self::ServerError { error }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }
}

pub trait NexError: Debug + Display + Send {
    fn error_code(&self) -> ResultCode;
}

pub type NexResult<T> = Result<T, Box<dyn NexError>>;

impl From<Box<dyn NexError>> for Error {
    fn from(error: Box<dyn NexError>) -> Self {
        Self::Generic {
            message: error.to_string(),
        }
    }
}

impl<T: NexError> From<T> for Error {
    fn from(error: T) -> Self {
        Self::Generic {
            message: error.to_string(),
        }
    }
}
