use crate::server::Server;
use arc4::Arc4;

pub struct Client<'a> {
    server: &'a Server,
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
}

impl<'a> Client<'a> {
    pub fn get_server(&self) -> &'a Server {
        self.server
    }
}
