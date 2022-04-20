use crate::{
    client::ClientConnection,
    packet::PacketV1,
    result::{Error as NexError, NexResult},
    rmc::RMCRequest,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait EventHandler {
    async fn on_syn(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_connect(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_data(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_disconnect(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_ping(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        packet: &PacketV1,
    ) -> NexResult<()>;

    async fn on_rmc_request(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        rmc_request: &RMCRequest,
    ) -> NexResult<()>;
    async fn on_error(&self, error: NexError);
}
