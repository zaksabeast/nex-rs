use super::NexError;
use crate::server;
use snafu::Snafu;
use std::fmt::Debug;

/// The top level error for anything nex related.
/// All server errors and generic errors provided by nex protocol trait
/// implementations will eventually bubble up to this.
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

impl<T: NexError> From<T> for Error {
    fn from(error: T) -> Self {
        Self::Generic {
            message: error.to_string(),
        }
    }
}
