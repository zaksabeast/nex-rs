use super::{PacketFlags, PacketType};

#[derive(Debug)]
pub struct BasePacket {
    pub(super) data: Vec<u8>,
    pub(super) source: u8,
    pub(super) destination: u8,
    pub(super) packet_type: PacketType,
    pub(super) flags: PacketFlags,
    pub(super) session_id: u8,
    pub(super) signature: Vec<u8>,
    pub(super) sequence_id: u16,
    pub(super) connection_signature: Vec<u8>,
    pub(super) fragment_id: u8,
    pub(super) payload: Vec<u8>,
}

impl BasePacket {
    pub(super) fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }
}

impl Default for BasePacket {
    fn default() -> Self {
        Self {
            source: 0,
            destination: 0,
            session_id: 0,
            sequence_id: 0,
            fragment_id: 0,
            data: vec![],
            signature: vec![],
            connection_signature: vec![],
            payload: vec![],
            packet_type: PacketType::Connect,
            flags: PacketFlags::new(0),
        }
    }
}
