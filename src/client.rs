use crate::{counter::Counter, rc4::Rc4, server::Server};

pub struct ClientContext {
    pub cipher: Rc4,
    pub decipher: Rc4,
    pub flags_version: u32,
    pub signature_key: Vec<u8>,
    pub signature_base: u32,
    pub session_key: Vec<u8>,
}

impl Default for ClientContext {
    fn default() -> Self {
        Self {
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            flags_version: 1,
            signature_key: vec![],
            signature_base: 0,
            session_key: vec![],
        }
    }
}

pub struct Client {
    secure_key: Vec<u8>,
    server_connection_signature: Vec<u8>,
    client_connection_signature: Vec<u8>,
    session_id: u32,
    pid: u32,
    local_station_url: String,
    connection_id: u32,
    connected: bool,
    sequence_id_in: Counter,
    sequence_id_out: Counter,
    context: ClientContext,
}

impl Client {
    pub fn new(server: &mut Server) -> Self {
        Self {
            secure_key: vec![],
            server_connection_signature: vec![],
            client_connection_signature: vec![],
            session_id: 0,
            pid: 0,
            local_station_url: "".to_string(),
            connection_id: 0,
            connected: false,
            sequence_id_in: Counter::default(),
            sequence_id_out: Counter::default(),
            context: ClientContext {
                cipher: Rc4::new(&[0]),
                decipher: Rc4::new(&[0]),
                flags_version: server.get_flags_version(),
                signature_key: vec![],
                signature_base: 0,
                session_key: vec![],
            },
        }
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
