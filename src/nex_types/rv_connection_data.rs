use super::NexString;
use no_std_io::{Cursor, EndianRead, EndianWrite, Error};

#[derive(Default, EndianRead, EndianWrite)]
pub struct RVConnectionData {
    station_url: NexString,
    // Should be Vec<u8>, but always empty
    special_protocols: u32,
    station_url_special_protocols: NexString,
    time: u64,
}
