use super::{PacketFlags, PacketType};
use crate::rmc_request::RMCRequest;

pub struct BasePacket {
    pub data: Vec<u8>,
    pub version: u8,
    pub source: u8,
    pub destination: u8,
    pub packet_type: PacketType,
    pub flags: PacketFlags,
    pub session_id: u8,
    pub signature: Vec<u8>,
    pub sequence_id: u16,
    pub connection_signature: Vec<u8>,
    pub fragment_id: u8,
    pub payload: Vec<u8>,
    pub rmc_request: RMCRequest,
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
