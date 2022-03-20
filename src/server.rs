use crate::{client::Client, counter::Counter, packet::Packet};

#[derive(Default)]
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

    fn listen(address: String) {
        unimplemented!()
    }

    fn handle_socket_message() -> Result<(), &'static str> {
        unimplemented!()
    }

    fn client_connected(client: &mut Client) -> bool {
        unimplemented!()
    }

    fn kick(client: &mut Client) {
        unimplemented!()
    }

    fn send_ping(client: &mut Client) {
        unimplemented!()
    }

    fn acknowledge_packet<'a>(packet: impl Packet<'a>, payload: Vec<u8>) {
        unimplemented!()
    }

    fn use_packet_compression(use_packet_compression: bool) {
        unimplemented!()
    }

    fn find_client_from_pid<'a>(pid: u32) -> &'a mut Client<'a> {
        unimplemented!()
    }

    fn send<'a>(packet: impl Packet<'a>) {
        unimplemented!()
    }

    fn send_fragment<'a>(packet: impl Packet<'a>, fragment_id: u32) {
        unimplemented!()
    }

    fn send_raw(conn: String, data: Vec<u8>) {
        unimplemented!()
    }
}
