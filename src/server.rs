use crate::{
    client::ClientConnection,
    compression::{dummy_compression, zlib_compression},
    counter::Counter,
    packet::{Packet, PacketFlag, PacketType, PacketV1},
    stream::StreamOut,
};
use no_std_io::StreamWriter;
use rand::RngCore;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;

pub trait EventHandler: Default {
    fn on_syn(&self, packet: &PacketV1) {}
    fn on_connect(&self, packet: &PacketV1) {}
    fn on_data(&self, packet: &PacketV1) {}
    fn on_disconnect(&self, packet: &PacketV1) {}
    fn on_ping(&self, packet: &PacketV1) {}
}

pub struct Server<Handler: EventHandler> {
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
    ping_kick_thread: Option<JoinHandle<()>>,
    clients: Arc<Mutex<Vec<ClientConnection>>>,
    handler: Handler,
}

impl<Handler: EventHandler> Default for Server<Handler> {
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
            ping_kick_thread: None,
            clients: Arc::new(Mutex::new(vec![])),
            handler: Handler::default(),
        }
    }
}

impl<Handler: EventHandler> Server<Handler> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_nex_version(&mut self, nex_version: u32) {
        self.nex_version = nex_version;
    }

    pub fn get_access_key(&self) -> String {
        self.access_key.to_string()
    }

    pub fn set_access_key(&mut self, access_key: String) {
        self.access_key = access_key;
    }

    pub fn get_checksum_version(&self) -> u32 {
        self.checksum_version
    }

    pub fn get_flags_version(&self) -> u32 {
        self.flags_version
    }

    pub fn get_prudp_version(&self) -> u32 {
        self.prudp_version
    }

    pub async fn listen(&mut self, addr: &str) -> Result<(), &'static str> {
        let socket = UdpSocket::bind(addr)
            .await
            .map_err(|_| "Couldn't bind to address")?;
        self.socket = Some(socket);

        let clients = Arc::clone(&self.clients);
        self.ping_kick_thread = Some(tokio::spawn(async move {
            let mut invertal = time::interval(Duration::from_secs(3));
            invertal.tick().await;

            loop {
                invertal.tick().await;
                let mut clients = clients.lock().await;

                for client in clients.iter_mut() {
                    if let Some(timer) = client.get_kick_timer() {
                        client.set_kick_timer(Some(timer.saturating_sub(3)));
                    }
                }

                *clients = clients
                    .iter()
                    .filter_map(|c| {
                        if c.get_kick_timer() == Some(0) {
                            None
                        } else {
                            Some(c.clone())
                        }
                    })
                    .collect::<Vec<ClientConnection>>();
            }
        }));

        loop {
            let result = self.handle_socket_message().await;
            if result.is_err() {
                println!("Error {:?}", result);
            }
        }
    }

    async fn handle_socket_message(&self) -> Result<(), &'static str> {
        let mut buf: Vec<u8> = vec![0; 0x200];
        let socket = match &self.socket {
            Some(socket) => Ok(socket),
            None => Err("No socket"),
        }?;

        let (receive_size, peer) = socket
            .recv_from(&mut buf)
            .await
            .map_err(|_| "UDP Receive error")?;

        buf.resize(receive_size, 0);

        let mut clients = self.clients.lock().await;

        let found_client = clients
            .iter_mut()
            .find(|client| client.get_address() == peer);

        let client = match found_client {
            Some(client) => client,
            None => {
                let mut new_client = ClientConnection::new(peer, self);
                new_client.update_access_key(self.get_access_key());
                clients.push(new_client);
                // We just pushed a client, so we know one exists
                clients.last_mut().unwrap()
            }
        };

        let packet = client.new_packet(buf)?;

        self.increase_ping_timeout_time(client);

        let flags = packet.get_flags();
        if flags.ack() || flags.multi_ack() {
            return Ok(());
        }

        let packet_type = packet.get_packet_type();

        match packet_type {
            PacketType::Syn => {
                client.reset();
                client.set_is_connected(true);
                client.set_kick_timer(Some(self.ping_timeout));
                self.handler.on_syn(&packet);
            }
            PacketType::Connect => {
                let client_connection_signature = packet.get_connection_signature().to_vec();
                client.set_client_connection_signature(client_connection_signature);
                client.update_access_key(self.get_access_key());
                self.handler.on_connect(&packet);
            }
            PacketType::Disconnect => {
                self.kick(peer).await;
                self.handler.on_disconnect(&packet);
            }
            PacketType::Data => {
                self.handler.on_data(&packet);
            }
            PacketType::Ping => {
                self.handler.on_ping(&packet);
            }
        };

        if flags.needs_ack()
            && (packet_type != PacketType::Connect
            || (packet_type == PacketType::Connect && packet.get_payload().is_empty()))
        {
            self.acknowledge_packet(&packet, None, client).await?;
        }

        Ok(())
    }

    async fn check_if_client_connected(&mut self, client: &ClientConnection) -> bool {
        self.clients
            .lock()
            .await
            .iter()
            .any(|item| item.get_address() == client.get_address())
    }

    async fn kick(&self, addr: SocketAddr) {
        let mut clients = self.clients.lock().await;
        let client_index = clients
            .iter_mut()
            .position(|client| client.get_address() == addr);

        if let Some(index) = client_index {
            clients.remove(index);
        }
    }

    async fn send_ping(&mut self, client: &mut ClientConnection) -> Result<(), &'static str> {
        let mut packet = client.new_packet(vec![])?;

        packet.set_source(0xa1);
        packet.set_destination(0xaf);
        packet.set_packet_type(PacketType::Ping);
        packet.set_flags(PacketFlag::Ack | PacketFlag::Reliable);

        self.send(client, &mut packet).await?;
        Ok(())
    }

    async fn acknowledge_packet(
        &self,
        packet: &PacketV1,
        payload: Option<Vec<u8>>,
        client: &mut ClientConnection,
    ) -> Result<(), &'static str> {
        let mut ack_packet = client.new_packet(vec![])?;

        ack_packet.set_source(packet.get_destination());
        ack_packet.set_destination(packet.get_source());
        ack_packet.set_packet_type(packet.get_packet_type());
        ack_packet.set_sequence_id(packet.get_sequence_id());
        ack_packet.set_fragment_id(packet.get_fragment_id());
        ack_packet.set_flags(PacketFlag::Ack | PacketFlag::HasSize);

        if let Some(payload) = payload {
            if !payload.is_empty() {
                ack_packet.set_payload(payload);
            }
        }

        match ack_packet.get_packet_type() {
            PacketType::Syn => {
                let mut connection_signature = vec![0; 16];
                rand::thread_rng().fill_bytes(&mut connection_signature);
                client.set_server_connection_signature(connection_signature.clone());
                ack_packet.set_connection_signature(connection_signature);
                ack_packet.set_supported_functions(packet.get_supported_functions());
                ack_packet.set_maximum_substream_id(0);
            }
            PacketType::Connect => {
                ack_packet.set_connection_signature(vec![0; 16]);
                ack_packet.set_supported_functions(packet.get_supported_functions());
                ack_packet.set_initial_sequence_id(10000);
                ack_packet.set_maximum_substream_id(0);
            }
            PacketType::Data => {
                // Aggregate acknowledgement
                ack_packet.get_mut_flags().clear_flag(PacketFlag::Ack);
                ack_packet.get_mut_flags().set_flag(PacketFlag::MultiAck);

                let mut payload_stream = StreamOut::new();

                // New version
                if self.nex_version >= 2 {
                    ack_packet.set_sequence_id(0);
                    ack_packet.set_substream_id(1);

                    // We're going to mimic nex-go and do one ack packet
                    payload_stream.checked_write_stream(&0u8); // substream id
                    payload_stream.checked_write_stream(&0u8); // length of additional sequence ids
                    payload_stream.checked_write_stream_le(&packet.get_sequence_id());
                }

                ack_packet.set_payload(payload_stream.into())
            }
            _ => {}
        };

        ack_packet.set_substream_id(0);

        let encoded_packet = &client.encode_packet(&mut ack_packet);
        self.send_raw(client.get_address(), encoded_packet).await?;

        Ok(())
    }

    async fn send(
        &mut self,
        client: &mut ClientConnection,
        packet: &mut PacketV1,
    ) -> Result<(), &'static str> {
        let fragment_size: usize = self.fragment_size.into();
        let data = packet.get_payload().to_vec();
        let fragment_count = data.len() / fragment_size;
        let mut fragment_data = data.as_slice();

        for i in 0..fragment_count {
            let fragment_id: u8 = (i + 1).try_into().map_err(|_| "Too many fragments!")?;

            if fragment_data.len() < fragment_size {
                packet.set_payload(fragment_data.to_vec());
                self.send_fragment(client, packet, fragment_id).await?;
            } else {
                packet.set_payload(data[..fragment_size].to_vec());
                self.send_fragment(client, packet, fragment_id).await?;
                fragment_data = &data[fragment_size..];
            }
        }

        Ok(())
    }

    async fn send_fragment(
        &mut self,
        client: &mut ClientConnection,
        packet: &mut PacketV1,
        fragment_id: u8,
    ) -> Result<usize, &'static str> {
        let compressed_data = self.compress_packet(packet.get_payload());
        let sequence_id = client
            .increment_sequence_id_out()
            .try_into()
            .expect("Sequence Id does not fit into u16");

        packet.set_sequence_id(sequence_id);
        packet.set_fragment_id(fragment_id);
        packet.set_payload(compressed_data);

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
        if self.use_packet_compression {
            zlib_compression::compress(data)
        } else {
            dummy_compression::compress(data)
        }
    }

    fn increase_ping_timeout_time(&self, client: &mut ClientConnection) {
        client.set_kick_timer(Some(self.ping_timeout));
    }
}
