use crate::{crypt, packet::PacketType};
use snafu::Snafu;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu(display(
      "Invalid crypt operation: {}",
      error.to_string()
  ))]
    CryptError { error: crypt::Error },
    #[snafu(display(
        "Invalid packet read for PacketType::{:?}, sequence_id: 0x{:02x}: {}",
        packet_type,
        sequence_id,
        message,
    ))]
    InvalidPacketRead {
        packet_type: PacketType,
        sequence_id: u16,
        message: String,
    },
    #[snafu(display("Error: {}", message))]
    Generic { message: String },
}

impl From<crypt::Error> for Error {
    fn from(error: crypt::Error) -> Self {
        Self::CryptError { error }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }
}

pub type ClientConnectionResult<T> = Result<T, Error>;
