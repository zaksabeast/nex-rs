use crate::counter::Counter;
use crate::{
    client::{ClientConnection, ClientContext},
    compression::{dummy_compression, zlib_compression},
    packet::{Packet, PacketFlag, PacketType, PacketV1},
    rmc::RMCRequest,
};
use async_trait::async_trait;
use getset::{CopyGetters, Getters, Setters};
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

    async fn on_rmc_request(
        &self,
        client: &mut ClientConnection,
        rmc_request: &RMCRequest,
    ) -> Result<(), &'static str>;
}

#[derive(Debug, Getters, CopyGetters, Setters)]
#[getset(skip)]
pub struct ServerSettings {
    #[getset(set = "pub")]
    access_key: String,
    #[getset(set = "pub")]
    nex_version: u32,
    #[getset(set = "pub")]
    prudp_version: u32,
    #[getset(set = "pub")]
    fragment_size: u16,
    flags_version: u32,
    use_packet_compression: bool,
    #[getset(set = "pub")]
    ping_timeout: u32,
    checksum_version: u32,
}

impl ServerSettings {
    pub fn create_client_context(&self) -> ClientContext {
        ClientContext::new(self.flags_version, self.prudp_version, &self.access_key)
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            access_key: "".to_string(),
            nex_version: 0,
            use_packet_compression: false,
            prudp_version: 1,
            fragment_size: 1300,
            ping_timeout: 5,
            flags_version: 1,
            checksum_version: 1,
        }
    }
}

#[derive(Default)]
pub struct BaseServer {
    settings: ServerSettings,
    socket: Option<UdpSocket>,
    pub connection_id_counter: Counter,
    ping_kick_thread: Option<JoinHandle<()>>,
    clients: Arc<Mutex<Vec<ClientConnection>>>,
}

impl BaseServer {
    pub fn new(settings: ServerSettings) -> Self {
        Self {
            settings,
            socket: None,
            connection_id_counter: Counter::new(10),
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

    fn set_fragment_size(&mut self, fragment_size: u16) {
        self.get_mut_base().settings.fragment_size = fragment_size;
    }

    fn set_ping_timeout(&mut self, ping_timeout: u32) {
        self.get_mut_base().settings.ping_timeout = ping_timeout;
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

    async fn receive_data(&self) -> Result<(Vec<u8>, SocketAddr), &'static str> {
        let mut buf: Vec<u8> = vec![0; 0x1000];
        let socket = match &self.get_base().socket {
            Some(socket) => Ok(socket),
            None => Err("No socket"),
        }?;

        let (receive_size, peer) = socket
            .recv_from(&mut buf)
            .await
            .map_err(|_| "UDP Receive error")?;

        buf.resize(receive_size, 0);

        Ok((buf, peer))
    }

    async fn emit_packet_events(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        match packet.get_packet_type() {
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

                if client.can_decode_rmc_request(&packet) {
                    let rmc_request = client.decode_rmc_request(&packet)?;
                    self.on_rmc_request(client, &rmc_request).await?;
                }
            }
            PacketType::Ping => {
                self.on_ping(client, &packet).await?;
            }
        };

        Ok(())
    }

    fn should_ignore_packet(&self, client: &mut ClientConnection, packet: &PacketV1) -> bool {
        let packet_type = packet.get_packet_type();

        // Ignore packets from disconnected clients
        if !client.is_connected() && packet_type != PacketType::Syn {
            return true;
        }

        // Ignore packets we've already handled
        if packet_type != PacketType::Ping && packet.get_sequence_id() < client.get_sequence_id_in()
        {
            return true;
        }

        return false;
    }

    fn proces_connection_init(&self, client: &mut ClientConnection, packet: &PacketV1) {
        match packet.get_packet_type() {
            PacketType::Syn => {
                client.reset();
                client.set_is_connected(true);
                client.set_kick_timer(Some(self.get_base().settings.ping_timeout));

                let mut connection_signature = vec![0; 16];
                rand::thread_rng().fill_bytes(&mut connection_signature);
                client.set_server_connection_signature(connection_signature.clone());
            }
            PacketType::Connect => {
                let client_connection_signature = packet.get_connection_signature().to_vec();
                client.set_client_connection_signature(client_connection_signature);
            }
            _ => {}
        }
    }

    fn increment_sequence_id_in(&self, client: &mut ClientConnection, packet: &PacketV1) {
        // Pings have their own sequence ids
        if packet.get_packet_type() != PacketType::Ping {
            client.increment_sequence_id_in();
        }
    }

    async fn handle_disconnect(&self, client: &mut ClientConnection, packet: &PacketV1) {
        if packet.get_packet_type() == PacketType::Disconnect {
            let addr = client.get_address();
            self.kick(addr).await;
        }
    }

    fn find_or_create_client<'a>(
        &self,
        clients: &'a mut Vec<ClientConnection>,
        addr: SocketAddr,
    ) -> &'a mut ClientConnection {
        let client_index = clients
            .iter()
            .position(|client| client.get_address() == addr);

        match client_index {
            Some(index) => &mut clients[index],
            None => {
                let settings = &self.get_base().settings;
                let new_client = ClientConnection::new(addr, settings.create_client_context());
                clients.push(new_client);
                // We just pushed a client, so we know one exists
                clients.last_mut().unwrap()
            }
        }
    }

    async fn handle_socket_message(&self) -> Result<(), &'static str> {
        let base = self.get_base();
        let (buf, peer) = self.receive_data().await?;

        let client_mutex = &base.clients;
        let mut clients = client_mutex.lock().await;
        let client = self.find_or_create_client(&mut clients, peer);

        let packet = client.read_packet(buf)?;

        if self.should_ignore_packet(client, &packet) {
            return Ok(());
        }

        client.set_kick_timer(Some(base.settings.ping_timeout));

        if self.accept_acknowledge_packet(&packet) {
            return Ok(());
        }

        self.proces_connection_init(client, &packet);
        self.acknowledge_packet(client, &packet).await?;
        self.emit_packet_events(client, &packet).await?;
        self.increment_sequence_id_in(client, &packet);
        self.handle_disconnect(client, &packet).await;

        Ok(())
    }

    async fn kick(&self, addr: SocketAddr) {
        let client_mutex = self.get_clients();
        let mut clients = client_mutex.lock().await;
        let client_index = clients
            .iter_mut()
            .position(|potential_client| potential_client.get_address() == addr);

        if let Some(index) = client_index {
            clients.remove(index);
        }
    }

    async fn send_ping(&self, client: &mut ClientConnection) -> Result<(), &'static str> {
        self.send(client, PacketV1::new_ping_packet()).await
    }

    fn accept_acknowledge_packet(&self, packet: &PacketV1) -> bool {
        let flags = packet.get_flags();
        if flags.ack() || flags.multi_ack() {
            // TODO: actually handle ack packets
            return true;
        }

        return false;
    }

    async fn acknowledge_packet(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let packet_type = packet.get_packet_type();
        let flags = packet.get_flags();
        let payload = packet.get_payload();

        if flags.needs_ack()
            && (packet_type != PacketType::Connect
                || (packet_type == PacketType::Connect && payload.is_empty()))
        {
            let nex_version = self.get_base().settings.nex_version;
            self.send_acknowledge_packet(&packet, client, nex_version, None)
                .await?;
        }

        Ok(())
    }

    async fn send_acknowledge_packet(
        &self,
        packet: &PacketV1,
        client: &mut ClientConnection,
        nex_version: u32,
        payload: Option<Vec<u8>>,
    ) -> Result<(), &'static str> {
        let mut ack_packet = packet.new_ack_packet();

        if let Some(payload) = payload {
            if !payload.is_empty() {
                ack_packet.set_payload(payload);
            }
        }

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
        self.send_raw(client, encoded_packet).await?;

        Ok(())
    }

    async fn send_success<MethodId: Into<u32>, Data: Into<Vec<u8>>>(
        &self,
        client: &mut ClientConnection,
        protocol_id: u8,
        method_id: MethodId,
        call_id: u32,
        data: Data,
    ) -> Result<(), &'static str> {
        let packet = client.new_rmc_success(protocol_id, method_id, call_id, data);
        self.send(client, packet).await
    }

    async fn send_error<MethodId: Into<u32>>(
        &self,
        client: &mut ClientConnection,
        protocol_id: u8,
        method_id: MethodId,
        call_id: u32,
        error_code: u32,
    ) -> Result<(), &'static str> {
        let packet = client.new_rmc_error(protocol_id, method_id, call_id, error_code);
        self.send(client, packet).await
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
        let sequence_id = client.increment_sequence_id_out();

        packet.set_sequence_id(sequence_id);
        packet.set_fragment_id(fragment_id);
        packet.set_payload(compressed_data);

        let encoded_packet = client.encode_packet(packet);
        self.send_raw(client, &encoded_packet).await
    }

    async fn send_raw(
        &self,
        client: &ClientConnection,
        data: &[u8],
    ) -> Result<usize, &'static str> {
        let socket = match &self.get_base().socket {
            Some(socket) => Ok(socket),
            None => Err("No socket"),
        }?;
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
