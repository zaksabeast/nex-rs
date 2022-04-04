use super::NexString;
use no_std_io::{EndianRead, EndianWrite};

#[derive(Default, EndianRead, EndianWrite)]
pub struct RVConnectionData {
    pub station_url: NexString,
    // Should be Vec<u8>, but always empty
    pub special_protocols: u32,
    pub station_url_special_protocols: NexString,
    pub time: u64,
}
