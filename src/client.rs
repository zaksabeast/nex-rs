use crate::{counter::Counter, server::Server};
use arc4::Arc4;

pub struct Client<'a> {
    server: &'a mut Server,
    cipher: Arc4<'a>,
    decipher: Arc4<'a>,
    signature_key: Vec<u8>,
    signature_base: u32,
    secure_key: Vec<u8>,
    server_connection_signature: Vec<u8>,
    client_connection_signature: Vec<u8>,
    session_id: u32,
    session_key: Vec<u8>,
    pid: u32,
    local_station_url: String,
    connection_id: u32,
    connected: bool,
    sequence_id_in: Counter,
    sequence_id_out: Counter,
}

impl<'a> Client<'a> {
    pub fn new(server: &'a mut Server) -> Self {
        Self {
            server,
            cipher: Arc4::with_key(&[0]),
            decipher: Arc4::with_key(&[0]),
            signature_key: vec![],
            signature_base: 0,
            secure_key: vec![],
            server_connection_signature: vec![],
            client_connection_signature: vec![],
            session_id: 0,
            session_key: vec![],
            pid: 0,
            local_station_url: "".to_string(),
            connection_id: 0,
            connected: false,
            sequence_id_in: Counter::default(),
            sequence_id_out: Counter::default(),
        }
    }

    pub fn get_server(&self) -> &Server {
        self.server
    }

    pub fn set_nex_version(&mut self, nex_version: u32) {
        self.server.set_nex_version(nex_version);
    }

    pub fn get_cipher(&mut self) -> &mut Arc4<'a> {
        &mut self.cipher
    }

    pub fn get_decipher(&mut self) -> &mut Arc4<'a> {
        &mut self.decipher
    }

    pub fn get_signature_key(&self) -> &[u8] {
        &self.signature_key
    }

    pub fn get_signature_base(&self) -> u32 {
        self.signature_base
    }

    pub fn get_session_key(&self) -> &[u8] {
        &self.session_key
    }

    fn reset(&mut self) {
        unimplemented!();
    }

    fn get_address(&self) -> String {
        unimplemented!();
    }

    fn update_rc4_key(&mut self, rc4_key: Vec<u8>) {
        unimplemented!();
    }

    fn update_access_key(&mut self, access_key: String) {
        unimplemented!();
    }

    fn increase_ping_timeout_time(&mut self, seconds: u32) {
        unimplemented!();
    }

    fn start_timeout_timer(&mut self) {
        unimplemented!();
    }
}
