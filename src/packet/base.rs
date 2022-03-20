use super::{PacketFlags, PacketType};
use crate::rmc_request::RMCRequest;

pub struct BasePacket {
    pub(super) data: Vec<u8>,
    pub(super) version: u8,
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
    pub(super) rmc_request: RMCRequest,
}

impl BasePacket {
    pub(super) fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            version: 0,
            source: 0,
            destination: 0,
            packet_type: PacketType::Connect,
            flags: PacketFlags::new(0),
            session_id: 0,
            signature: vec![],
            sequence_id: 0,
            connection_signature: vec![],
            fragment_id: 0,
            payload: vec![],
            rmc_request: RMCRequest::default(),
        }
    }
}
