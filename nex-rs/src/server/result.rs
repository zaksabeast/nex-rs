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
}

pub type ServerResult<T> = Result<T, Error>;
