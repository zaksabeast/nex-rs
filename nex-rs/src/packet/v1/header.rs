use super::packet::PacketV1;
use crate::packet::{Packet, PacketFlags, PacketType};

#[derive(Debug)]
pub struct PacketV1Header {
    pub(super) magic: u16,
    pub(super) version: u8,
    pub(super) options_length: u8,
    pub(super) payload_size: u16,
    pub(super) source: u8,
    pub(super) destination: u8,
    pub(super) packet_type: PacketType,
    pub(super) flags: PacketFlags,
    pub(super) session_id: u8,
    pub(super) substream_id: u8,
    pub(super) sequence_id: u16,
}

impl Default for PacketV1Header {
    fn default() -> Self {
        Self {
            magic: 0xd0ea,
            version: PacketV1::VERSION,
            options_length: 0,
            payload_size: 0,
            source: PacketV1::SERVER_ID,
            destination: PacketV1::CLIENT_ID,
            packet_type: PacketType::Syn,
            flags: PacketFlags::new(0),
            session_id: 0,
            substream_id: 0,
            sequence_id: 0,
        }
    }
}
