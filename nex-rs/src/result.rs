use crate::server;
use snafu::Snafu;

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

pub type NexResult<T> = Result<T, Error>;
