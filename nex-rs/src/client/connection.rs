use super::{ClientConnectionResult, ClientContext, Error};
use crate::{
    crypto::rc4::Rc4,
    packet::{Packet, PacketResult, PacketV1},
    rmc::{RMCRequest, RMCResponse},
};
use no_std_io::Reader;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct ClientConnection {
    address: SocketAddr,
    session_id: u8,
    pid: u32,
    is_connected: bool,
    kick_timer: u32,
    context: ClientContext,
}

impl ClientConnection {
    pub fn new(address: SocketAddr, context: ClientContext, kick_timer: u32) -> Self {
        Self {
            address,
            session_id: 0,
            pid: 0,
            is_connected: true,
            kick_timer,
            context,
        }
    }

    pub fn flags_version(&self) -> u32 {
        self.context.flags_version
    }

    pub fn encode_packet(&mut self, packet: &mut PacketV1) -> Vec<u8> {
        self.context.encrypt_packet(packet);
        packet.to_bytes(&self.context.signature_context)
    }

    pub fn validate_packet(&mut self, packet: &PacketV1) -> PacketResult<()> {
        packet.validate(&self.context.signature_context)
    }

    pub fn new_data_packet(&self, payload: Vec<u8>) -> PacketV1 {
        PacketV1::new_data_packet(
            self.session_id,
            self.context
                .signature_context
                .client_connection_signature()
                .to_vec(),
            payload,
            self.context.flags_version,
        )
    }

    pub fn new_rmc_success(
        &self,
        protocol_id: u8,
        method_id: impl Into<u32>,
        call_id: u32,
        data: impl Into<Vec<u8>>,
    ) -> PacketV1 {
        let rmc_response = RMCResponse::new_success(protocol_id, method_id, call_id, data.into());
        self.new_data_packet(rmc_response.into())
    }

    pub fn new_rmc_error(
        &self,
        protocol_id: u8,
        method_id: impl Into<u32>,
        call_id: u32,
        error_code: u32,
    ) -> PacketV1 {
        let rmc_response = RMCResponse::new_error(protocol_id, method_id, call_id, error_code);
        self.new_data_packet(rmc_response.into())
    }

    pub fn get_session_id(&self) -> u8 {
        self.session_id
    }

    pub fn set_session_key(&mut self, key: Vec<u8>) {
        self.context.signature_context.set_session_key(key);
    }

    pub fn set_client_connection_signature(&mut self, client_connection_signature: Vec<u8>) {
        self.context
            .signature_context
            .set_client_connection_signature(client_connection_signature);
    }

    pub fn get_client_connection_signature(&mut self) -> &[u8] {
        self.context.signature_context.client_connection_signature()
    }

    pub fn set_server_connection_signature(&mut self, server_connection_signature: Vec<u8>) {
        self.context
            .signature_context
            .set_server_connection_signature(server_connection_signature);
    }

    pub fn get_server_connection_signature(&self) -> &[u8] {
        self.context.signature_context.server_connection_signature()
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn set_is_connected(&mut self, is_connected: bool) {
        self.is_connected = is_connected;
    }

    pub fn get_address(&self) -> SocketAddr {
        self.address
    }

    pub fn get_pid(&self) -> u32 {
        self.pid
    }

    pub fn set_pid(&mut self, pid: u32) {
        self.pid = pid;
    }

    pub fn get_mut_context(&mut self) -> &mut ClientContext {
        &mut self.context
    }

    pub fn get_sequence_id_in(&self) -> u16 {
        self.context.get_sequence_id_in()
    }

    pub fn increment_sequence_id_in(&mut self) -> u16 {
        self.context.increment_sequence_id_in()
    }

    pub fn increment_sequence_id_out(&mut self) -> u16 {
        self.context.increment_sequence_id_out()
    }

    pub fn update_rc4_key(&mut self, rc4_key: &[u8]) {
        self.context.cipher = Rc4::new(rc4_key);
        self.context.decipher = Rc4::new(rc4_key);
    }

    pub fn get_kick_timer(&self) -> u32 {
        self.kick_timer
    }

    pub fn set_kick_timer(&mut self, seconds: u32) {
        self.kick_timer = seconds;
    }

    pub fn decrement_kick_timer(&mut self, seconds: u32) {
        self.kick_timer = self.kick_timer.saturating_sub(seconds);
    }

    pub fn can_decode_rmc_request(&self, packet: &PacketV1) -> bool {
        self.context.can_decrypt_packet(packet).is_ok()
    }

    pub fn decode_rmc_request(&mut self, packet: &PacketV1) -> ClientConnectionResult<RMCRequest> {
        let payload = self.context.decrypt_packet(packet)?;
        payload.read_le(0).map_err(|_| Error::InvalidPacketRead {
            packet_type: packet.get_packet_type(),
            sequence_id: packet.get_sequence_id(),
            message: "Cannot read rmc request from payload".into(),
        })
    }
}
