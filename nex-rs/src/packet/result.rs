use super::PacketType;
use snafu::Snafu;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu(display(
        "Invalid packet size: wanted 0x{:x}, but received 0x{:x}.  Context: {}",
        wanted_size,
        received_size,
        context
    ))]
    InvalidSize {
        wanted_size: usize,
        received_size: usize,
        context: &'static str,
    },
    #[snafu(display("Invalid magic 0x{:04x}", magic))]
    InvalidMagic { magic: u16 },
    #[snafu(display("Invalid version 0x{:02x}", version))]
    InvalidVersion { version: u8 },
    #[snafu(display("Invalid packet type 0x{:04x}", packet_type))]
    InvalidPacketType { packet_type: u16 },
    #[snafu(display("Invalid packet option 0x{:02x}", option))]
    InvalidPacketOption { option: u8 },
    #[snafu(display(
        "Invalid signature: calculated: {:02x?}, found: {:02x?} for PacketType::{:?} and sequence_id 0x{:02x}",
        calculated_signature,
        found_signature,packet_type,
        sequence_id
    ))]
    InvalidSignature {
        calculated_signature: Vec<u8>,
        found_signature: Vec<u8>,
        packet_type: PacketType,
        sequence_id: u16,
    },
    #[snafu(display("Error reading or writing packet: {}", error))]
    IoError { error: no_std_io::Error },
}

pub type PacketResult<T> = Result<T, Error>;

impl From<no_std_io::Error> for Error {
    fn from(error: no_std_io::Error) -> Self {
        Self::IoError { error }
    }
}
