mod packet_flag;
mod packet_type;
mod v1;

use crate::{client::Client, rmc_request::RMCRequest};

pub struct Packet<'a> {
    sender: &'a Client<'a>,
    data: Vec<u8>,
    version: u8,
    source: u8,
    destination: u8,
    packet_type: u16,
    flags: u16,
    session_id: u8,
    signature: Vec<u8>,
    sequence_id: u16,
    connection_signature: Vec<u8>,
    fragment_id: u8,
    payload: Vec<u8>,
    rmc_request: RMCRequest,
}

impl<'a> Packet<'a> {
    fn new(client: &'a Client<'a>, data: Vec<u8>) -> Self {
        Self {
            data,
            sender: client,
            version: 0,
            source: 0,
            destination: 0,
            packet_type: 0,
            flags: 0,
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
