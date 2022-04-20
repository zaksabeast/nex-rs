use super::ServerSettings;
use crate::{client, client::ClientConnection, counter::Counter};
use std::sync::Arc;
use tokio::{net::UdpSocket, sync::Mutex, task::JoinHandle};

#[derive(Default)]
pub struct BaseServer<ClientData: client::ClientData> {
    pub connection_id_counter: Counter,
    pub(super) settings: ServerSettings,
    pub(super) socket: Option<UdpSocket>,
    pub(super) ping_kick_thread: Option<JoinHandle<()>>,
    pub(super) clients: Arc<Mutex<Vec<ClientConnection<ClientData>>>>,
}

impl<ClientData: client::ClientData> BaseServer<ClientData> {
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
