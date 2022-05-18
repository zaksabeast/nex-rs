use crate::{client::ClientConnection, rmc::RMCRequest, server::ServerResult};

#[async_trait::async_trait]
pub trait Route<const PROTOCOL_ID: u8, const METHOD_ID: u32> {
    async fn run(&self, client: &mut ClientConnection, request: &RMCRequest) -> ServerResult<()>;
}
