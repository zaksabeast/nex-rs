use crate::{
    counter::Counter,
    packet::{Packet, PacketV1},
    rc4::Rc4,
};
use getset::{CopyGetters, Getters};
use std::net::SocketAddr;

#[derive(Clone, CopyGetters, Getters)]
#[getset(skip)]
pub struct ClientContext {
    #[getset(get_copy = "pub")]
    flags_version: u32,
    #[getset(get_copy = "pub")]
    signature_base: u32,
    #[getset(get = "pub")]
    server_connection_signature: Vec<u8>,
    #[getset(get = "pub")]
    client_connection_signature: Vec<u8>,
    #[getset(get = "pub")]
    signature_key: [u8; 16],
    #[getset(get = "pub")]
    session_key: Vec<u8>,
    cipher: Rc4,
    decipher: Rc4,
    prudp_version: u32,
}

impl ClientContext {
    pub fn new(flags_version: u32, prudp_version: u32, access_key: &str) -> Self {
        Self {
            flags_version,
            prudp_version,
            signature_key: crate::md5::hash(access_key.as_bytes()),
            signature_base: access_key.as_bytes().iter().map(|byte| *byte as u32).sum(),
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            server_connection_signature: vec![],
            client_connection_signature: vec![],
            session_key: vec![],
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        self.cipher.encrypt(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        self.decipher.decrypt(data)
    }
}

impl Default for ClientContext {
    fn default() -> Self {
        Self {
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            flags_version: 1,
            prudp_version: 1,
            server_connection_signature: vec![],
            client_connection_signature: vec![],
            signature_key: [0; 16],
            signature_base: 0,
            session_key: vec![],
        }
    }
}

#[derive(Clone)]
pub struct ClientConnection {
    address: SocketAddr,
    secure_key: Vec<u8>,
    session_id: u8,
    pid: u32,
    local_station_url: String,
    connection_id: u32,
    is_connected: bool,
    sequence_id_in: Counter,
    sequence_id_out: Counter,
    kick_timer: Option<u32>,
    context: ClientContext,
}

impl ClientConnection {
    pub fn new(address: SocketAddr, context: ClientContext) -> Self {
        Self {
            address,
            secure_key: vec![],
            session_id: 0,
            pid: 0,
            local_station_url: "".to_string(),
            connection_id: 0,
            is_connected: false,
            sequence_id_in: Counter::default(),
            sequence_id_out: Counter::default(),
            kick_timer: None,
            context,
        }
    }

    pub fn encode_packet(&mut self, packet: &mut PacketV1) -> Vec<u8> {
        packet.to_bytes(&mut self.context)
    }

    pub fn read_packet(&mut self, data: Vec<u8>) -> Result<PacketV1, &'static str> {
        PacketV1::read_packet(&mut self.context, data)
    }

    pub fn new_data_packet(&self, payload: Vec<u8>) -> PacketV1 {
        PacketV1::new_data_packet(
            self.session_id,
            self.context.client_connection_signature.to_vec(),
            payload,
        )
    }

    pub fn get_session_id(&self) -> u8 {
        self.session_id
    }

    pub fn set_session_key(&mut self, key: Vec<u8>) {
        self.context.session_key = key;
    }

    pub fn set_client_connection_signature(&mut self, client_connection_signature: Vec<u8>) {
        self.context.client_connection_signature = client_connection_signature;
    }

    pub fn get_client_connection_signature(&mut self) -> &[u8] {
        &self.context.client_connection_signature
    }

    pub fn set_server_connection_signature(&mut self, server_connection_signature: Vec<u8>) {
        self.context.server_connection_signature = server_connection_signature;
    }

    pub fn get_server_connection_signature(&mut self) -> &[u8] {
        &self.context.server_connection_signature
    }

    pub fn set_is_connected(&mut self, is_connected: bool) {
        self.is_connected = is_connected;
    }

    pub fn reset(&mut self) {
        self.sequence_id_in = Counter::default();
        self.sequence_id_out = Counter::default();

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

    pub fn get_mut_context(&mut self) -> &mut ClientContext {
        &mut self.context
    }

    pub fn increment_sequence_id_out(&mut self) -> u16 {
        self.sequence_id_out.increment()
    }

    fn update_rc4_key(&mut self, rc4_key: &[u8]) {
        self.context.cipher = Rc4::new(rc4_key);
        self.context.decipher = Rc4::new(rc4_key);
    }

    pub fn get_kick_timer(&self) -> Option<u32> {
        self.kick_timer
    }

    pub fn set_kick_timer(&mut self, seconds: Option<u32>) {
        self.kick_timer = seconds;
    }
}
