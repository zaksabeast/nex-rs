use crate::{
    client::Client,
    counter::Counter,
    packet::{Packet, PacketFlag, PacketType, PacketV1},
};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

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
    clients: Vec<Client>,
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
            clients: vec![],
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
        let socket = UdpSocket::bind(addr)
            .await
            .map_err(|_| "Couldn't bind to address")?;
        self.socket = Some(socket);

        loop {
            let result = self.handle_socket_message().await;
            if result.is_err() {
                println!("Error {:?}", result);
            }
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
            .await
            .map_err(|_| "UDP Receive error")?;

        let found_client = self
            .clients
            .iter_mut()
            .find(|client| client.get_address() == peer);

        let client = match found_client {
            Some(client) => client,
            None => {
                let new_client = Client::new(peer, self);
                self.clients.push(new_client);
                // We just pushed a client, so we know one exists
                self.clients.last_mut().unwrap()
            }
        };

        let packet = client.new_packet(buf)?;

        client.increase_ping_timeout_time(self.ping_timeout);

        let base = packet.get_base();
        if base.flags.ack() || base.flags.multi_ack() {
            return Ok(());
        }

        match base.packet_type {
            PacketType::Syn => {
                client.reset();
                client.set_is_connected(true);
                client.start_timeout_timer();
            }
            PacketType::Connect => {
                client.set_client_connection_signature(base.connection_signature.clone());
            }

            PacketType::Disconnect => {
                self.kick(peer);
            }
            _ => {}
        };

        if base.packet_type == PacketType::Disconnect && base.flags.needs_ack() {
            if base.packet_type != PacketType::Connect
                || (base.packet_type == PacketType::Connect && base.payload.len() <= 0)
            {
                self.acknowledge_packet(packet, None, peer).await?;
            }
        }

        Ok(())
    }

    fn check_if_client_connected(&mut self, client: &Client) -> bool {
        self.clients
            .iter()
            .any(|item| item.get_address() == client.get_address())
    }

    fn kick(&mut self, addr: SocketAddr) {
        let client_index = self
            .clients
            .iter_mut()
            .position(|client| client.get_address() == addr);

        if let Some(index) = client_index {
            self.clients.remove(index);
        }
    }

    fn send_ping(&mut self, client: &mut Client) -> Result<(), &'static str> {
        let mut packet = client.new_packet(vec![])?;
        let base = packet.get_mut_base();
        base.source = 0xa1;
        base.destination = 0xaf;
        base.packet_type = PacketType::Ping;
        base.flags |= PacketFlag::Ack;
        base.flags |= PacketFlag::Reliable;

        self.send(packet);
        Ok(())
    }

    async fn acknowledge_packet(
        &mut self,
        packet: PacketV1,
        payload: Option<Vec<u8>>,
        client_addr: SocketAddr,
    ) -> Result<(), &'static str> {
        let found_client = self
            .clients
            .iter_mut()
            .find(|client| client.get_address() == client_addr);

        if let Some(client) = found_client {
            let client_base = packet.get_base();
            let mut ack_packet = client.new_packet(vec![])?;
            let mut ack_base = ack_packet.get_mut_base();
            ack_base.source = client_base.destination;
            ack_base.destination = client_base.source;
            ack_base.packet_type = client_base.packet_type;
            ack_base.sequence_id = client_base.sequence_id;
            ack_base.fragment_id = client_base.fragment_id;
            ack_base.flags |= PacketFlag::Ack;
            ack_base.flags |= PacketFlag::HasSize;

            if let Some(payload) = payload {
                if payload.len() > 0 {
                    ack_base.payload = payload;
                }
            }

            match ack_base.packet_type {
                PacketType::Syn | PacketType::Connect | PacketType::Data => {
                    unimplemented!()
                }
                _ => {}
            };

            ack_packet.substream_id = 0;

            let encoded_packet = &client.encode_packet(ack_packet);
            self.send_raw(client_addr, encoded_packet).await?;
        }

        Ok(())
    }

    fn use_packet_compression(&mut self, use_packet_compression: bool) {
        unimplemented!()
    }

    fn find_client_from_pid(&mut self, pid: u32) -> Option<&mut Client> {
        self.clients
            .iter_mut()
            .find(|client| client.get_pid() == pid)
    }

    fn send(&mut self, packet: PacketV1) {
        unimplemented!()
    }

    async fn send_fragment(
        &mut self,
        client: &mut Client,
        mut packet: PacketV1,
        fragment_id: u8,
    ) -> Result<usize, &'static str> {
        let compressed_data = self.compress_packet(&packet.get_base().payload);

        let base = packet.get_mut_base();
        base.fragment_id = fragment_id;
        base.payload = compressed_data;
        base.sequence_id = client
            .increment_sequence_id_out()
            .try_into()
            .expect("Sequence Id does not fit into u16");

        let encoded_packet = client.encode_packet(packet);
        self.send_raw(client.get_address(), &encoded_packet).await
    }

    async fn send_raw(&self, peer: SocketAddr, data: &[u8]) -> Result<usize, &'static str> {
        self.socket
            .as_ref()
            .expect("Socket not found")
            .send_to(data, &peer)
            .await
            .map_err(|_| "Error sending data")
    }

    fn compress_packet(&self, data: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
