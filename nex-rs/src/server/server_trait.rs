use super::{BaseServer, ClientMap, Error, EventHandler, ServerResult};
use crate::{
    client::ClientConnection,
    packet::{Packet, PacketFlag, PacketType, PacketV1},
    result::NexResult,
};
use async_trait::async_trait;
use no_std_io::{StreamContainer, StreamWriter};
use rand::RngCore;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, sync::RwLock, time};

#[async_trait]
pub trait Server: EventHandler {
    fn get_base(&self) -> &BaseServer;
    fn get_mut_base(&mut self) -> &mut BaseServer;

    fn get_clients(&self) -> Arc<RwLock<ClientMap>> {
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
    fn set_flags_version(&mut self, flags_version: u32) {
        self.get_mut_base().settings.flags_version = flags_version;
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
        let ping_kick_thread = tokio::spawn(async move {
            let mut invertal = time::interval(Duration::from_secs(3));
            invertal.tick().await;

            loop {
                invertal.tick().await;

                // We can't use iterator-like methods since each client has a lock.
                // We could have a new list and re-add each item, but we'll add almost
                // every client each time.
                // A kick list means iterating over a map, then a list, but we'll almost never
                // iterate over the kick list.
                let mut kick_list = vec![];
                let clients = clients_lock.read().await;

                for (addr, client_lock) in clients.iter() {
                    let mut client = client_lock.write().await;
                    if client.get_kick_timer() == 0 || !client.is_connected() {
                        kick_list.push(*addr);
                    } else {
                        client.decrement_kick_timer(3);
                    }
                }

                drop(clients);
                let mut clients = clients_lock.write().await;
                for kick_addr in kick_list.iter() {
                    clients.remove(kick_addr);
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
            let clone = Arc::clone(&server);
            tokio::spawn(async move {
                if let Err(error) = clone.handle_socket_message(buf, peer).await {
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
            PacketType::Invalid => {}
        };

        Ok(())
    }

    fn should_ignore_packet(&self, client: &mut ClientConnection, packet: &PacketV1) -> bool {
        let packet_type = packet.get_packet_type();

        // Ignore packets from disconnected clients
        if !client.is_connected() && packet_type != PacketType::Syn {
            return true;
        }

        // Ignore packets we're not expecting
        if packet_type != PacketType::Ping
            && packet.get_sequence_id() != client.get_sequence_id_in()
        {
            return true;
        }

        false
    }

    fn handle_connection_init(&self, client: &mut ClientConnection, packet: &PacketV1) {
        match packet.get_packet_type() {
            PacketType::Syn => {
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
            self.kick(client).await;
        }
    }

    async fn handle_socket_message(&self, message: Vec<u8>, peer: SocketAddr) -> NexResult<()> {
        let settings = &self.get_base().settings;
        let packet = PacketV1::read_packet(message, self.get_flags_version())?;
        let clients_lock = self.get_clients();

        if packet.get_packet_type() == PacketType::Syn {
            let mut clients = clients_lock.write().await;
            clients.insert(
                peer,
                RwLock::new(ClientConnection::new(
                    peer,
                    settings.create_client_context(),
                    settings.ping_timeout,
                )),
            );
        }

        let clients = clients_lock.read().await;

        if let Some(client) = clients.get(&peer) {
            return self.handle_packet(packet, client).await;
        }

        Ok(())
    }

    async fn handle_packet(
        &self,
        packet: PacketV1,
        client_lock: &RwLock<ClientConnection>,
    ) -> NexResult<()> {
        let base = self.get_base();
        let mut client = client_lock.write().await;
        client.set_kick_timer(base.settings.ping_timeout);
        client.validate_packet(&packet)?;

        if self.should_ignore_packet(&mut client, &packet) {
            return Ok(());
        }

        if self.accept_acknowledge_packet(&packet) {
            return Ok(());
        }

        self.handle_connection_init(&mut client, &packet);
        self.acknowledge_packet(&mut client, &packet).await?;
        self.emit_packet_events(&mut client, &packet).await?;
        self.increment_sequence_id_in(&mut client, &packet);
        self.handle_disconnect(&mut client, &packet).await;

        Ok(())
    }

    async fn kick(&self, client: &mut ClientConnection) {
        client.set_is_connected(false);
    }

    async fn send_ping(&self, client: &mut ClientConnection) -> ServerResult<()> {
        self.send(client, PacketV1::new_ping_packet(client.flags_version()))
            .await
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
                ack_packet.set_supported_functions(packet.flags_version());
                ack_packet.set_maximum_substream_id(0);
            }
            PacketType::Connect => {
                ack_packet.set_connection_signature(vec![0; 16]);
                ack_packet.set_supported_functions(packet.flags_version());
                ack_packet.set_initial_sequence_id(10000);
                ack_packet.set_maximum_substream_id(0);
            }
            PacketType::Data => {
                // Aggregate acknowledgement
                ack_packet.set_flags(PacketFlag::MultiAck | PacketFlag::HasSize);

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
