use crate::{
    client,
    client::ClientConnection,
    packet::PacketV1,
    result::{Error as NexError, NexResult},
    rmc::RMCRequest,
};
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler<ClientData: client::ClientData> {
    async fn on_syn(
        &self,
        client: &mut ClientConnection<ClientData>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_connect(
        &self,
        client: &mut ClientConnection<ClientData>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_data(
        &self,
        client: &mut ClientConnection<ClientData>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_disconnect(
        &self,
        client: &mut ClientConnection<ClientData>,
        packet: &PacketV1,
    ) -> NexResult<()>;
    async fn on_ping(
        &self,
        client: &mut ClientConnection<ClientData>,
        packet: &PacketV1,
    ) -> NexResult<()>;

    async fn on_rmc_request(
        &self,
        client: &mut ClientConnection<ClientData>,
        rmc_request: &RMCRequest,
    ) -> NexResult<()>;
    async fn on_error(&self, error: NexError);
}
