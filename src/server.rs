use crate::{
    client::ClientConnection,
    compression::{dummy_compression, zlib_compression},
    packet::{Packet, PacketFlag, PacketType, PacketV1},
};
use async_trait::async_trait;
use no_std_io::{StreamContainer, StreamWriter};
use rand::RngCore;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time;

#[async_trait(?Send)]
pub trait EventHandler {
    async fn on_syn(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str>;
    async fn on_connect(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str>;
    async fn on_data(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str>;
    async fn on_disconnect(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str>;
    async fn on_ping(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str>;
}

pub struct ServerSettings {
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
}

impl ServerSettings {
    pub fn get_access_key(&self) -> String {
        self.access_key.to_string()
    }

    pub fn get_flags_version(&self) -> u32 {
        self.flags_version
    }

    pub fn get_prudp_version(&self) -> u32 {
        self.prudp_version
    }

    pub fn set_nex_version(&mut self, nex_version: u32) {
        self.nex_version = nex_version;
    }
}

impl Default for ServerSettings {
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
        }
    }
}

#[derive(Default)]
pub struct BaseServer {
    settings: ServerSettings,
    socket: Option<UdpSocket>,
    ping_kick_thread: Option<JoinHandle<()>>,
    clients: Arc<Mutex<Vec<ClientConnection>>>,
}

impl BaseServer {
    fn new(settings: ServerSettings) -> Self {
        Self {
            settings,
            socket: None,
            ping_kick_thread: None,
            clients: Arc::new(Mutex::new(vec![])),
        }
    }
}

#[async_trait(?Send)]
pub trait Server: EventHandler {
    fn get_base(&self) -> &BaseServer;
    fn get_mut_base(&mut self) -> &mut BaseServer;

    fn get_clients(&self) -> Arc<Mutex<Vec<ClientConnection>>> {
        Arc::clone(&self.get_base().clients)
    }

    fn get_access_key(&self) -> String {
        self.get_base().settings.access_key.to_string()
    }

    fn set_access_key(&mut self, access_key: String) {
        self.get_mut_base().settings.access_key = access_key;
    }

    fn set_nex_version(&mut self, nex_version: u32) {
        self.get_mut_base().settings.nex_version = nex_version;
    }

    fn get_checksum_version(&self) -> u32 {
        self.get_base().settings.checksum_version
    }

    fn get_flags_version(&self) -> u32 {
        self.get_base().settings.flags_version
    }

    fn get_prudp_version(&self) -> u32 {
        self.get_base().settings.prudp_version
    }

    async fn listen(&mut self, addr: &str) -> Result<(), &'static str> {
        let socket = UdpSocket::bind(addr)
            .await
            .map_err(|_| "Couldn't bind to address")?;

        self.get_mut_base().socket = Some(socket);

        let clients = Arc::clone(&self.get_base().clients);
        let ping_kick_thread = tokio::spawn(async move {
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
        });

        self.get_mut_base().ping_kick_thread = Some(ping_kick_thread);

        loop {
            let result = self.handle_socket_message().await;
            if result.is_err() {
                println!("Error {:?}", result);
            }
        }
    }

    async fn handle_socket_message(&self) -> Result<(), &'static str> {
        let mut buf: Vec<u8> = vec![0; 0x200];
        let base = self.get_base();
        let socket = match &base.socket {
            Some(socket) => Ok(socket),
            None => Err("No socket"),
        }?;

        let (receive_size, peer) = socket
            .recv_from(&mut buf)
            .await
            .map_err(|_| "UDP Receive error")?;

        buf.resize(receive_size, 0);

        let client_mutex = &base.clients;
        let mut clients = client_mutex.lock().await;

        let found_client = clients
            .iter_mut()
            .find(|client| client.get_address() == peer);

        let client = match found_client {
            Some(client) => client,
            None => {
                let settings = &base.settings;
                let mut new_client = ClientConnection::new(peer, settings);
                new_client.update_access_key(self.get_access_key());
                clients.push(new_client);
                // We just pushed a client, so we know one exists
                clients.last_mut().unwrap()
            }
        };

        let packet = client.new_packet(buf)?;

        client.set_kick_timer(Some(base.settings.ping_timeout));

        let flags = packet.get_flags();
        if flags.ack() || flags.multi_ack() {
            return Ok(());
        }

        let packet_type = packet.get_packet_type();

        match packet_type {
            PacketType::Syn => {
                client.reset();
                client.set_is_connected(true);
                client.set_kick_timer(Some(base.settings.ping_timeout));

                let mut connection_signature = vec![0; 16];
                rand::thread_rng().fill_bytes(&mut connection_signature);
                client.set_server_connection_signature(connection_signature.clone());
            }
            PacketType::Connect => {
                let client_connection_signature = packet.get_connection_signature().to_vec();
                client.set_client_connection_signature(client_connection_signature);
                client.update_access_key(self.get_access_key());
            }
            _ => {}
        }

        if flags.needs_ack()
            && (packet_type != PacketType::Connect
                || (packet_type == PacketType::Connect && packet.get_payload().is_empty()))
        {
            let nex_version = self.get_base().settings.nex_version;
            Self::acknowledge_packet(socket, &packet, client, nex_version, None).await?;
        }

        match packet_type {
            PacketType::Syn => {
                self.on_syn(client, &packet).await?;
            }
            PacketType::Connect => {
                self.on_connect(client, &packet).await?;
            }
            PacketType::Disconnect => {
                self.on_disconnect(client, &packet).await?;
            }
            PacketType::Data => {
                self.on_data(client, &packet).await?;
            }
            PacketType::Ping => {
                self.on_ping(client, &packet).await?;
            }
        };

        if packet_type == PacketType::Disconnect {
            let addr = client.get_address();
            Self::kick(&mut clients, addr);
        }

        Ok(())
    }

    fn kick(clients: &mut Vec<ClientConnection>, addr: SocketAddr) {
        let client_index = clients
            .iter_mut()
            .position(|potential_client| potential_client.get_address() == addr);

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

        self.send(client, packet).await?;
        Ok(())
    }

    async fn acknowledge_packet(
        socket: &UdpSocket,
        packet: &PacketV1,
        client: &mut ClientConnection,
        nex_version: u32,
        payload: Option<Vec<u8>>,
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
        ack_packet.set_substream_id(0);
        ack_packet.set_version(1);

        match ack_packet.get_packet_type() {
            PacketType::Syn => {
                ack_packet
                    .set_connection_signature(client.get_server_connection_signature().to_vec());
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

                let mut payload_stream = StreamContainer::new(vec![]);

                // New version
                if nex_version >= 2 {
                    ack_packet.set_sequence_id(0);
                    ack_packet.set_substream_id(1);

                    // We're going to mimic nex-go and do one ack packet
                    payload_stream.checked_write_stream_le(&0u8); // substream id
                    payload_stream.checked_write_stream_le(&0u8); // length of additional sequence ids
                    payload_stream.checked_write_stream_le(&packet.get_sequence_id());
                }

                ack_packet.set_payload(payload_stream.into_raw())
            }
            _ => {}
        };

        let encoded_packet = &client.encode_packet(&mut ack_packet);
        Self::send_raw(socket, client, encoded_packet).await?;

        Ok(())
    }

    async fn send(
        &self,
        client: &mut ClientConnection,
        mut packet: PacketV1,
    ) -> Result<(), &'static str> {
        let fragment_size: usize = self.get_base().settings.fragment_size.into();
        let data = packet.get_payload().to_vec();
        let fragment_count = data.len() / fragment_size;
        let mut fragment_data = data.as_slice();
        let packet = &mut packet;

        for i in 0..=fragment_count {
            let fragment_id: u8 = (i + 1).try_into().map_err(|_| "Too many fragments!")?;

            if fragment_data.len() < fragment_size {
                packet.set_payload(fragment_data.to_vec());
                // Last fragment is always 0
                self.send_fragment(client, packet, 0).await?;
            } else {
                packet.set_payload(data[..fragment_size].to_vec());
                self.send_fragment(client, packet, fragment_id).await?;
                fragment_data = &data[fragment_size..];
            }
        }

        Ok(())
    }

    async fn send_fragment(
        &self,
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
        let socket = match &self.get_base().socket {
            Some(socket) => Ok(socket),
            None => Err("No socket"),
        }?;
        Self::send_raw(socket, client, &encoded_packet).await
    }

    async fn send_raw(
        socket: &UdpSocket,
        client: &ClientConnection,
        data: &[u8],
    ) -> Result<usize, &'static str> {
        socket
            .send_to(data, client.get_address())
            .await
            .map_err(|_| "Error sending data")
    }

    fn compress_packet(&self, data: &[u8]) -> Vec<u8> {
        if self.get_base().settings.use_packet_compression {
            zlib_compression::compress(data)
        } else {
            dummy_compression::compress(data)
        }
    }
}
