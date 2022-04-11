use crate::{
    counter::Counter,
    crypt_result::{self, CryptResult},
    packet::{Packet, PacketResult, PacketType, PacketV1, SignatureContext},
    rc4::Rc4,
    rmc::{RMCRequest, RMCResponse},
};
use getset::{CopyGetters, Getters};
use no_std_io::Reader;
use snafu::Snafu;
use std::net::SocketAddr;

#[derive(Debug, PartialEq, Snafu)]
pub enum Error {
    #[snafu(display(
        "Invalid crypt operation: {}",
        error.to_string()
    ))]
    CryptError { error: crypt_result::Error },
    #[snafu(display(
        "Invalid packet read for PacketType::{:?}, sequence_id: 0x{:02x}: {}",
        packet_type,
        sequence_id,
        message,
    ))]
    InvalidPacketRead {
        packet_type: PacketType,
        sequence_id: u16,
        message: String,
    },
    #[snafu(display("Error: {}", message))]
    Generic { message: String },
}

impl From<crypt_result::Error> for Error {
    fn from(error: crypt_result::Error) -> Self {
        Self::CryptError { error }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }
}

pub type ClientConnectionResult<T> = Result<T, Error>;

#[derive(Clone, CopyGetters, Getters)]
#[getset(skip)]
pub struct ClientContext {
    #[getset(get_copy = "pub")]
    flags_version: u32,
    #[getset(get_copy = "pub")]
    signature_base: u32,
    cipher: Rc4,
    decipher: Rc4,
    prudp_version: u32,
    sequence_id_in: Counter,
    sequence_id_out: Counter,
    signature_context: SignatureContext,
}

impl ClientContext {
    pub fn new(flags_version: u32, prudp_version: u32, access_key: &str) -> Self {
        Self {
            flags_version,
            prudp_version,
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            signature_context: SignatureContext::new(access_key),
            ..Default::default()
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> CryptResult<Vec<u8>> {
        self.cipher.encrypt(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> CryptResult<Vec<u8>> {
        self.decipher.decrypt(data)
    }

    pub fn get_sequence_id_in(&self) -> u16 {
        self.sequence_id_in
            .value()
            .try_into()
            .expect("Sequence id out does not fit into u16")
    }

    pub fn increment_sequence_id_in(&mut self) -> u16 {
        self.sequence_id_in
            .increment()
            .try_into()
            .expect("Sequence id out does not fit into u16")
    }

    pub fn increment_sequence_id_out(&mut self) -> u16 {
        self.sequence_id_out
            .increment()
            .try_into()
            .expect("Sequence id out does not fit into u16")
    }

    fn can_decrypt_packet(&self, packet: &PacketV1) -> ClientConnectionResult<()> {
        if packet.get_packet_type() != PacketType::Data {
            return Err("Only data packets can have payloads".into());
        }

        if packet.get_flags().multi_ack() {
            return Err("Ack packets can not hold payloads".into());
        }

        if packet.get_sequence_id() != self.get_sequence_id_in() {
            return Err("Tried to decode a packet out of order".into());
        }

        Ok(())
    }

    fn decrypt_packet(&mut self, packet: &PacketV1) -> ClientConnectionResult<Vec<u8>> {
        self.can_decrypt_packet(packet)?;
        self.decipher
            .decrypt(packet.get_payload())
            .map_err(|error| error.into())
    }

    fn can_encrypt_packet(&self, packet: &PacketV1) -> Result<(), &'static str> {
        if packet.get_packet_type() != PacketType::Data {
            return Err("Only data packets can have payloads");
        }

        if packet.get_flags().multi_ack() {
            return Err("Ack packets can not hold payloads");
        }

        if packet.get_payload().is_empty() {
            return Err("Cannot encode an empty payload");
        }

        Ok(())
    }

    fn encrypt_packet(&mut self, packet: &mut PacketV1) {
        if self.can_encrypt_packet(packet).is_ok() {
            let payload = self.cipher.encrypt(packet.get_payload()).unwrap();
            packet.set_payload(payload);
        }
    }
}

impl Default for ClientContext {
    fn default() -> Self {
        Self {
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            flags_version: 1,
            prudp_version: 1,
            signature_base: 0,
            sequence_id_in: Counter::default(),
            sequence_id_out: Counter::default(),
            signature_context: SignatureContext::default(),
        }
    }
}

#[derive(Clone)]
pub struct ClientConnection {
    address: SocketAddr,
    session_id: u8,
    pid: u32,
    is_connected: bool,
    kick_timer: Option<u32>,
    context: ClientContext,
}

impl ClientConnection {
    pub fn new(address: SocketAddr, context: ClientContext) -> Self {
        Self {
            address,
            session_id: 0,
            pid: 0,
            is_connected: false,
            kick_timer: None,
            context,
        }
    }

    pub fn encode_packet(&mut self, packet: &mut PacketV1) -> Vec<u8> {
        self.context.encrypt_packet(packet);
        packet.to_bytes(self.context.flags_version, &self.context.signature_context)
    }

    pub fn read_packet(&mut self, data: Vec<u8>) -> PacketResult<PacketV1> {
        PacketV1::read_packet(
            data,
            self.context.flags_version,
            &self.context.signature_context,
        )
    }

    pub fn new_data_packet(&self, payload: Vec<u8>) -> PacketV1 {
        PacketV1::new_data_packet(
            self.session_id,
            self.context
                .signature_context
                .client_connection_signature()
                .to_vec(),
            payload,
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

    pub fn get_server_connection_signature(&mut self) -> &[u8] {
        self.context.signature_context.server_connection_signature()
    }

    pub fn is_connected(&mut self) -> bool {
        self.is_connected
    }

    pub fn set_is_connected(&mut self, is_connected: bool) {
        self.is_connected = is_connected;
    }

    pub fn reset(&mut self) {
        self.context.sequence_id_in = Counter::default();
        self.context.sequence_id_out = Counter::default();

        self.update_rc4_key(b"CD&ML");

        if self.context.prudp_version == 0 {
            self.set_client_connection_signature(vec![0; 4]);
            self.set_server_connection_signature(vec![0; 4]);
        } else {
            self.set_client_connection_signature(vec![]);
            self.set_server_connection_signature(vec![]);
        }

        self.set_is_connected(false);
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

    pub fn get_sequence_id_in(&mut self) -> u16 {
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

    pub fn get_kick_timer(&self) -> Option<u32> {
        self.kick_timer
    }

    pub fn set_kick_timer(&mut self, seconds: Option<u32>) {
        self.kick_timer = seconds;
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
