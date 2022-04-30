use super::{BaseServer, Error, EventHandler, ServerResult};
use crate::{
    client::{ClientConnection, Error::Generic},
    packet::{Packet, PacketFlag, PacketType, PacketV1},
    result::NexResult,
};
use async_trait::async_trait;
use no_std_io::{StreamContainer, StreamWriter};
use rand::RngCore;
use std::{collections::VecDeque, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, sync::RwLock, time};

#[async_trait]
pub trait Server: EventHandler {
    fn get_base(&self) -> &BaseServer;
    fn get_mut_base(&mut self) -> &mut BaseServer;

    fn get_clients(&self) -> Arc<RwLock<Vec<RwLock<ClientConnection>>>> {
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

    fn get_socket(&self) -> ServerResult<&UdpSocket> {
        self.get_base().socket.as_ref().ok_or(Error::NoSocket)
    }

    async fn initialize(&mut self, addr: &str) -> ServerResult<()> {
        let socket = UdpSocket::bind(addr)
            .await
            .map_err(|_| Error::CouldNoBindToAddress)?;

        self.get_mut_base().socket = Some(socket);

        let clients_lock = Arc::clone(&self.get_base().clients);
        let packet_queues_lock = Arc::clone(&self.get_base().packet_queues);
        let ping_kick_thread = tokio::spawn(async move {
            let mut invertal = time::interval(Duration::from_secs(3));
            invertal.tick().await;

            loop {
                invertal.tick().await;

                let mut clients_guard = clients_lock.write().await;

                let len = clients_guard.len();

                let old_clients = std::mem::replace(&mut *clients_guard, Vec::with_capacity(len));

                for client_lock in old_clients.into_iter() {
                    let mut client = client_lock.write().await;
                    if let Some(timer) = client.get_kick_timer() {
                        if timer != 0 {
                            client.set_kick_timer(Some(timer.saturating_sub(3)));
                            drop(client);
                            clients_guard.push(client_lock);
                        } else {
                            packet_queues_lock
                                .write()
                                .await
                                .remove(&client.get_address());
                        }
                    } else {
                        drop(client);
                        clients_guard.push(client_lock);
                    }
                }
            }
        });

        self.get_mut_base().ping_kick_thread = Some(ping_kick_thread);

        Ok(())
    }

    async fn listen<T: Server + Sized + Send + Sync + 'static>(
        mut server: T,
        addr: &str,
    ) -> ServerResult<()> {
        server.initialize(addr).await?;
        let server = Arc::new(server);

        loop {
            let (buf, peer) = server.receive_data().await?;
            server
                .get_base()
                .packet_queues
                .write()
                .await
                .entry(peer)
                .or_insert_with(VecDeque::new)
                .push_back(buf);
            let clone = Arc::clone(&server);
            tokio::spawn(async move {
                if let Err(error) = clone.handle_socket_message(peer).await {
                    clone.on_error(error).await;
                }
            });
        }
    }

    async fn receive_data(&self) -> ServerResult<(Vec<u8>, SocketAddr)> {
        let mut buf: Vec<u8> = vec![0; 0x1000];
        let socket = &self.get_base().socket.as_ref().ok_or(Error::NoSocket)?;

        let (receive_size, peer) = socket
            .recv_from(&mut buf)
            .await
            .map_err(|_| Error::DataReceiveError)?;

        buf.resize(receive_size, 0);

        Ok((buf, peer))
    }

    async fn emit_packet_events(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> NexResult<()> {
        match packet.get_packet_type() {
            PacketType::Syn => {
                self.on_syn(client, packet).await?;
            }
            PacketType::Connect => {
                self.on_connect(client, packet).await?;
            }
            PacketType::Disconnect => {
                self.on_disconnect(client, packet).await?;
            }
            PacketType::Data => {
                self.on_data(client, packet).await?;

                if client.can_decode_rmc_request(packet) {
                    let rmc_request = client.decode_rmc_request(packet)?;
                    self.on_rmc_request(client, &rmc_request).await?;
                }
            }
            PacketType::Ping => {
                self.on_ping(client, packet).await?;
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

        false
    }

    fn handle_connection_init(&self, client: &mut ClientConnection, packet: &PacketV1) {
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

    async fn handle_disconnect(&self, addr: SocketAddr, packet: &PacketV1) {
        if packet.get_packet_type() == PacketType::Disconnect {
            self.kick(addr).await;
        }
    }

    async fn find_client<'a>(
        &self,
        clients: &'a [RwLock<ClientConnection>],
        addr: SocketAddr,
    ) -> Option<&'a RwLock<ClientConnection>> {
        for client in clients.iter() {
            if client.read().await.get_address() == addr {
                return Some(client);
            }
        }
        None
    }

    fn create_client(
        &self,
        clients: &mut Vec<RwLock<ClientConnection>>,
        addr: SocketAddr,
    ) -> usize {
        let settings = &self.get_base().settings;
        let new_client = RwLock::new(ClientConnection::new(
            addr,
            settings.create_client_context(),
        ));
        clients.push(new_client);
        clients.len() - 1
    }

    async fn handle_socket_message(&self, peer: SocketAddr) -> NexResult<()> {
        let base = self.get_base();

        let client_list_rwlock = self.get_clients();

        let mut clients = client_list_rwlock.read().await;

        let mut client = self.find_client(&clients, peer).await;

        if client.is_none() {
            drop(clients);
            let index = {
                let mut clients = client_list_rwlock.write().await;
                self.create_client(&mut clients, peer)
            };
            clients = client_list_rwlock.read().await;
            client = Some(&clients[index]);
        }
        let mut client = client.unwrap().write().await;

        let message =
            if let Some(entry) = self.get_base().packet_queues.write().await.get_mut(&peer) {
                if let Some(message) = entry.pop_front() {
                    message
                } else {
                    return Err(Generic {
                        message: "Failed to find packet".to_string(),
                    }
                    .into());
                }
            } else {
                return Err(Generic {
                    message: "Failed to find packet".to_string(),
                }
                .into());
            };

        let packet = client.read_packet(message)?;

        if self.should_ignore_packet(&mut client, &packet) {
            return Ok(());
        }

        client.set_kick_timer(Some(base.settings.ping_timeout));

        if self.accept_acknowledge_packet(&packet) {
            return Ok(());
        }

        self.handle_connection_init(&mut client, &packet);
        self.acknowledge_packet(&mut client, &packet).await?;
        self.emit_packet_events(&mut client, &packet).await?;
        self.increment_sequence_id_in(&mut client, &packet);
        drop(client);
        drop(clients);
        self.handle_disconnect(peer, &packet).await;

        Ok(())
    }

    async fn kick(&self, addr: SocketAddr) {
        let client_rwlock = self.get_clients();
        let mut clients = client_rwlock.write().await;
        let mut client_index = None;
        for (i, client) in clients.iter().enumerate() {
            let client = client.read().await;
            if client.get_address() == addr {
                client_index = Some(i);
                self.get_base()
                    .packet_queues
                    .write()
                    .await
                    .remove(&client.get_address());
                break;
            }
        }

        if let Some(index) = client_index {
            clients.remove(index);
        }
    }

    async fn send_ping(&self, client: &mut ClientConnection) -> ServerResult<()> {
        self.send(client, PacketV1::new_ping_packet()).await
    }

    fn accept_acknowledge_packet(&self, packet: &PacketV1) -> bool {
        let flags = packet.get_flags();
        if flags.ack() || flags.multi_ack() {
            // TODO: actually handle ack packets
            return true;
        }

        false
    }

    async fn acknowledge_packet(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> NexResult<()> {
        let packet_type = packet.get_packet_type();
        let flags = packet.get_flags();
        let payload = packet.get_payload();

        if flags.needs_ack()
            && (packet_type != PacketType::Connect
                || (packet_type == PacketType::Connect && payload.is_empty()))
        {
            self.send_acknowledge_packet(packet, client, None).await?;
        }

        Ok(())
    }

    async fn send_acknowledge_packet(
        &self,
        packet: &PacketV1,
        client: &mut ClientConnection,
        payload: Option<Vec<u8>>,
    ) -> NexResult<()> {
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
                if self.get_base().settings.nex_version >= 2 {
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

    async fn send_success<MethodId: Into<u32> + Send, Data: Into<Vec<u8>> + Send>(
        &self,
        client: &mut ClientConnection,
        protocol_id: u8,
        method_id: MethodId,
        call_id: u32,
        data: Data,
    ) -> ServerResult<()> {
        let packet = client.new_rmc_success(protocol_id, method_id, call_id, data);
        self.send(client, packet).await
    }

    async fn send_error<MethodId: Into<u32> + Send>(
        &self,
        client: &mut ClientConnection,
        protocol_id: u8,
        method_id: MethodId,
        call_id: u32,
        error_code: u32,
    ) -> ServerResult<()> {
        let packet = client.new_rmc_error(protocol_id, method_id, call_id, error_code);
        self.send(client, packet).await
    }

    async fn send(&self, client: &mut ClientConnection, mut packet: PacketV1) -> ServerResult<()> {
        let fragment_size: usize = self.get_base().settings.fragment_size.into();
        let data = packet.get_payload().to_vec();
        let fragment_count = data.len() / fragment_size;
        let mut fragment_data = data.as_slice();
        let packet = &mut packet;

        for i in 0..=fragment_count {
            let fragment_id: u8 = (i + 1).try_into().map_err(|_| Error::TooManyFragments {
                client_addr: client.get_address(),
                fragment_id: i + 1,
                sequence_id: packet.get_sequence_id(),
            })?;

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
    ) -> ServerResult<usize> {
        let sequence_id = client.increment_sequence_id_out();

        packet.set_sequence_id(sequence_id);
        packet.set_fragment_id(fragment_id);

        let encoded_packet = client.encode_packet(packet);
        self.send_raw(client, &encoded_packet).await
    }

    async fn send_raw(&self, client: &ClientConnection, data: &[u8]) -> ServerResult<usize> {
        let socket = self.get_socket()?;
        socket
            .send_to(data, client.get_address())
            .await
            .map_err(|_| Error::DataSendError)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        client::ClientContext, packet::SignatureContext, result::Error as NexError, rmc::RMCRequest,
    };
    use std::net::{IpAddr, Ipv4Addr};

    #[derive(Default)]
    struct TestServer {
        base: BaseServer,
    }

    #[async_trait]
    impl EventHandler for TestServer {
        async fn on_syn(
            &self,
            _client: &mut ClientConnection,
            _packet: &PacketV1,
        ) -> NexResult<()> {
            Ok(())
        }
        async fn on_connect(
            &self,
            _client: &mut ClientConnection,
            _packet: &PacketV1,
        ) -> NexResult<()> {
            Ok(())
        }
        async fn on_data(
            &self,
            _client: &mut ClientConnection,
            _packet: &PacketV1,
        ) -> NexResult<()> {
            Ok(())
        }
        async fn on_disconnect(
            &self,
            _client: &mut ClientConnection,
            _packet: &PacketV1,
        ) -> NexResult<()> {
            Ok(())
        }
        async fn on_ping(
            &self,
            _client: &mut ClientConnection,
            _packet: &PacketV1,
        ) -> NexResult<()> {
            Ok(())
        }

        async fn on_rmc_request(
            &self,
            _client: &mut ClientConnection,
            _rmc_request: &RMCRequest,
        ) -> NexResult<()> {
            Ok(())
        }
        async fn on_error(&self, _error: NexError) {}
    }

    #[async_trait]
    impl Server for TestServer {
        fn get_base(&self) -> &BaseServer {
            &self.base
        }
        fn get_mut_base(&mut self) -> &mut BaseServer {
            &mut self.base
        }
    }

    async fn get_server_with_client(client: ClientConnection) -> TestServer {
        let server = TestServer::default();
        let client_rwlock = server.get_clients();
        let mut clients = client_rwlock.write().await;
        clients.push(RwLock::new(client));

        server
    }

    #[tokio::test]
    #[ntest::timeout(5000)]
    async fn should_not_deadlock_when_handling_disconnect_message() {
        // Set up client
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut client = ClientConnection::new(addr, ClientContext::default());
        client.set_is_connected(true);

        // Get server with client
        let server = get_server_with_client(client).await;

        // Set up packet
        let mut disconnect_packet = PacketV1::new_disconnect_packet();
        let mut flags = disconnect_packet.get_flags();
        // Prevent ack response from server for test
        flags.clear_flag(PacketFlag::NeedsAck);
        disconnect_packet.set_flags(flags);
        let packet_bytes = disconnect_packet.to_bytes(4, &SignatureContext::default());

        let mut queue = VecDeque::new();
        queue.push_back(packet_bytes);

        server
            .get_base()
            .packet_queues
            .write()
            .await
            .insert(addr, queue);

        // Handle disconnect
        server.handle_socket_message(addr).await.unwrap();

        let client_rwlock = server.get_clients();
        let clients = client_rwlock.read().await;
        assert_eq!(clients.len(), 0);
    }
}
