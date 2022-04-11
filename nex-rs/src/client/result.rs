use crate::{crypto, packet::PacketType};
use snafu::Snafu;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu(display(
      "Invalid crypto operation: {}",
      error.to_string()
  ))]
    CryptoError { error: crypto::Error },
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

impl From<crypto::Error> for Error {
    fn from(error: crypto::Error) -> Self {
        Self::CryptoError { error }
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
