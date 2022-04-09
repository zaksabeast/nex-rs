use nex_rs::nex_types::NexString;
use no_std_io::{EndianRead, EndianWrite};

#[derive(Default, EndianRead, EndianWrite)]
pub struct ConnectionData {
    station_url: NexString,
    connection_id: u32,
}
