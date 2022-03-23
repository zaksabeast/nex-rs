use crate::{
    counter::Counter,
    packet::{Packet, PacketV1},
    rc4::Rc4,
    server::Server,
};
use md5::{Digest, Md5};
use std::net::SocketAddr;

#[derive(Clone)]
pub struct ClientContext {
    pub access_key: String,
    pub cipher: Rc4,
    pub decipher: Rc4,
    pub flags_version: u32,
    pub prudp_version: u32,
    pub signature_key: Vec<u8>,
    pub signature_base: u32,
    pub session_key: Vec<u8>,
}

impl Default for ClientContext {
    fn default() -> Self {
        Self {
            access_key: String::new(),
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            flags_version: 1,
            prudp_version: 1,
            signature_key: vec![],
            signature_base: 0,
            session_key: vec![],
        }
    }
}

#[derive(Clone)]
pub struct ClientConnection {
    address: SocketAddr,
    secure_key: Vec<u8>,
    server_connection_signature: Vec<u8>,
    client_connection_signature: Vec<u8>,
    session_id: u32,
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
    pub fn new(address: SocketAddr, server: &Server) -> Self {
        Self {
            address,
            secure_key: vec![],
            server_connection_signature: vec![],
            client_connection_signature: vec![],
            session_id: 0,
            pid: 0,
            local_station_url: "".to_string(),
            connection_id: 0,
            is_connected: false,
            sequence_id_in: Counter::default(),
            sequence_id_out: Counter::default(),
            kick_timer: None,
            context: ClientContext {
                access_key: server.get_access_key(),
                cipher: Rc4::new(&[0]),
                decipher: Rc4::new(&[0]),
                flags_version: server.get_flags_version(),
                prudp_version: server.get_prudp_version(),
                signature_key: vec![],
                signature_base: 0,
                session_key: vec![],
            },
        }
    }

    pub fn encode_packet(&mut self, packet: &mut PacketV1) -> Vec<u8> {
        packet.to_bytes(&mut self.context)
    }

    pub fn new_packet(&mut self, data: Vec<u8>) -> Result<PacketV1, &'static str> {
        PacketV1::new(data, &mut self.context)
    }

    pub fn set_client_connection_signature(&mut self, client_connection_signature: Vec<u8>) {
        self.client_connection_signature = client_connection_signature;
    }

    pub fn set_server_connection_signature(&mut self, server_connection_signature: Vec<u8>) {
        self.server_connection_signature = server_connection_signature;
    }

    pub fn set_is_connected(&mut self, is_connected: bool) {
        self.is_connected = is_connected;
    }

    pub fn reset(&mut self) {
        self.sequence_id_in = Counter::default();
        self.sequence_id_out = Counter::default();

        self.update_access_key(self.context.access_key.to_string());
        self.update_rc4_key("CD&ML".as_bytes());

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

    pub fn increment_sequence_id_out(&mut self) -> usize {
        self.sequence_id_out.increment()
    }

    fn update_rc4_key(&mut self, rc4_key: &[u8]) {
        self.context.cipher = Rc4::new(rc4_key);
        self.context.decipher = Rc4::new(rc4_key);
    }

    fn update_access_key(&mut self, access_key: String) {
        self.context.signature_base = access_key.as_bytes().iter().map(|&i| i as u32).sum();
        let mut md5 = Md5::new();

        md5.update(access_key.as_bytes());

        self.context.signature_key = md5.finalize().to_vec();
    }

    pub fn get_kick_timer(&self) -> Option<u32> {
        self.kick_timer
    }

    pub fn set_kick_timer(&mut self, seconds: Option<u32>) {
        self.kick_timer = seconds;
    }
}
