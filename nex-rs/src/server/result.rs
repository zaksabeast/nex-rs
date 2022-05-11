use crate::{client, crypto, packet};
use snafu::Snafu;
use std::net::SocketAddr;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu()]
    NoSocket,
    #[snafu()]
    CouldNoBindToAddress,
    #[snafu()]
    DataReceiveError,
    #[snafu()]
    DataSendError,
    #[snafu(display(
        "Tried to send too many fragments to {}: sequence_id 0x{:02x}, fragment_id 0x{:x}",
        client_addr,
        sequence_id,
        fragment_id,
    ))]
    TooManyFragments {
        client_addr: SocketAddr,
        sequence_id: u16,
        fragment_id: usize,
    },
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

pub type ServerResult<T> = Result<T, Error>;
