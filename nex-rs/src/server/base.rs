use super::ServerSettings;
use crate::{client::ClientConnection, counter::Counter};
use std::{
    collections::{BTreeMap, VecDeque},
    net::SocketAddr,
    sync::Arc,
};
use tokio::{net::UdpSocket, sync::RwLock, task::JoinHandle};

#[derive(Default)]
pub struct BaseServer {
    pub connection_id_counter: Counter,
    pub(super) settings: ServerSettings,
    pub(super) socket: Option<UdpSocket>,
    pub(super) ping_kick_thread: Option<JoinHandle<()>>,
    pub(super) clients: Arc<RwLock<Vec<RwLock<ClientConnection>>>>,
    pub(super) packet_queues: Arc<RwLock<BTreeMap<SocketAddr, VecDeque<Vec<u8>>>>>,
}

impl BaseServer {
    pub fn new(settings: ServerSettings) -> Self {
        Self {
            settings,
            socket: None,
            connection_id_counter: Counter::new(10),
            ping_kick_thread: None,
            clients: Arc::new(RwLock::new(vec![])),
            packet_queues: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}
