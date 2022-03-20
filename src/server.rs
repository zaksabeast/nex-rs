use crate::{client::Client, counter::Counter, packet::Packet};

pub struct Server {
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

    fn listen(&mut self, address: String) {
        unimplemented!()
    }

    fn handle_socket_message(&mut self) -> Result<(), &'static str> {
        unimplemented!()
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

    fn acknowledge_packet<'a>(&mut self, packet: impl Packet<'a>, payload: Vec<u8>) {
        unimplemented!()
    }

    fn use_packet_compression(&mut self, use_packet_compression: bool) {
        unimplemented!()
    }

    fn find_client_from_pid<'a>(&self, pid: u32) -> &'a mut Client<'a> {
        unimplemented!()
    }

    fn send<'a>(&mut self, packet: impl Packet<'a>) {
        unimplemented!()
    }

    fn send_fragment<'a>(&mut self, packet: impl Packet<'a>, fragment_id: u32) {
        unimplemented!()
    }

    fn send_raw(&mut self, conn: String, data: Vec<u8>) {
        unimplemented!()
    }
}
