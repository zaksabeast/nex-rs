use crate::{
    client::ClientConnection, packet::PacketV1, result::Error as NexError, rmc::RMCRequest,
    server::ServerResult,
};
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler {
    async fn on_syn(&self, client: &mut ClientConnection, packet: &PacketV1) -> ServerResult<()>;
    async fn on_connect(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> ServerResult<()>;
    async fn on_data(&self, client: &mut ClientConnection, packet: &PacketV1) -> ServerResult<()>;
    async fn on_disconnect(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> ServerResult<()>;
    async fn on_ping(&self, client: &mut ClientConnection, packet: &PacketV1) -> ServerResult<()>;

    async fn on_rmc_request(
        &self,
        client: &mut ClientConnection,
        rmc_request: &RMCRequest,
    ) -> ServerResult<()>;
    async fn on_protocol_method(&self, method_name: String);
    async fn on_error(&self, error: &NexError);
}
