use crate::{client, crypto, packet, server};
use snafu::Snafu;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu(display(
        "Packet error: {}",
        error.to_string()
    ))]
    PacketError { error: packet::Error },
    #[snafu(display(
        "Crypt error: {}",
        error.to_string()
    ))]
    CryptError { error: crypto::Error },
    #[snafu(display(
        "Client connection error: {}",
        error.to_string()
    ))]
    ClientConectionError { error: client::Error },
    #[snafu(display(
        "Client connection error: {}",
        error.to_string()
    ))]
    ServerError { error: server::Error },
    #[snafu(display(
        "IO Error: {}",
        error.to_string()
    ))]
    IoError { error: no_std_io::Error },
    #[snafu(display("Error: {}", message))]
    Generic { message: String },
}

impl From<packet::Error> for Error {
    fn from(error: packet::Error) -> Self {
        Self::PacketError { error }
    }
}

impl From<crypto::Error> for Error {
    fn from(error: crypto::Error) -> Self {
        Self::CryptError { error }
    }
}

impl From<client::Error> for Error {
    fn from(error: client::Error) -> Self {
        Self::ClientConectionError { error }
    }
}

impl From<server::Error> for Error {
    fn from(error: server::Error) -> Self {
        Self::ServerError { error }
    }
}

impl From<no_std_io::Error> for Error {
    fn from(error: no_std_io::Error) -> Self {
        Self::IoError { error }
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
