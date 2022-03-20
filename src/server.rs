use crate::{client::Client, counter::Counter, packet::Packet};
use std::net::UdpSocket;

pub struct Server {
    socket: Option<UdpSocket>,
    access_key: String,
    prudp_version: u32,
    nex_version: u32,
    fragment_size: u16,
    use_packet_compression: bool,
    ping_timeout: u32,
    signature_version: u32,
    flags_version: u32,
    checksum_version: u32,
    kerberos_key_size: u32,
    kerberos_key_derivation: u32,
    server_version: u32,
    connection_id_counter: Counter,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            socket: None,
            access_key: "".to_string(),
            nex_version: 0,
            server_version: 0,
            use_packet_compression: false,
            prudp_version: 1,
            fragment_size: 1300,
            ping_timeout: 5,
            signature_version: 0,
            flags_version: 1,
            checksum_version: 1,
            kerberos_key_size: 32,
            kerberos_key_derivation: 0,
            connection_id_counter: Counter::default(),
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_nex_version(&mut self, nex_version: u32) {
        self.nex_version = nex_version;
    }

    pub fn get_checksum_version(&self) -> u32 {
        self.checksum_version
    }

    pub fn get_flags_version(&self) -> u32 {
        self.flags_version
    }

    async fn listen(&mut self, addr: &str) -> Result<(), &'static str> {
        let socket = UdpSocket::bind(addr).map_err(|_| "Couldn't bind to address")?;
        self.socket = Some(socket);

        loop {
            self.handle_socket_message().await?;
        }
    }

    async fn handle_socket_message(&mut self) -> Result<(), &'static str> {
        let mut buf: Vec<u8> = vec![];
        let socket = match &self.socket {
            Some(socket) => Ok(socket),
            None => Err("No socket"),
        }?;

        let (receive_size, peer) = socket
            .recv_from(&mut buf)
            .map_err(|_| "UDP Receive error")?;

        Ok(())
    }

    fn client_connected(&mut self, client: &mut Client) -> bool {
        unimplemented!()
    }

    fn kick(&mut self, client: &mut Client) {
        unimplemented!()
    }

    fn send_ping(&mut self, client: &mut Client) {
        unimplemented!()
    }

    fn acknowledge_packet(&mut self, packet: impl Packet, payload: Vec<u8>) {
        unimplemented!()
    }

    fn use_packet_compression(&mut self, use_packet_compression: bool) {
        unimplemented!()
    }

    fn find_client_from_pid(&mut self, pid: u32) -> &mut Client {
        unimplemented!()
    }

    fn send(&mut self, packet: impl Packet) {
        unimplemented!()
    }

    fn send_fragment(&mut self, packet: impl Packet, fragment_id: u32) {
        unimplemented!()
    }

    fn send_raw(&mut self, conn: String, data: Vec<u8>) {
        unimplemented!()
    }
}
