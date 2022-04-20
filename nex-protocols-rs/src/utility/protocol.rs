use async_trait::async_trait;
use nex_rs::{
    client, client::ClientConnection, nex_types::ResultCode, result::NexResult, rmc::RMCRequest,
    server::Server,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const UTILITY_PROTOCOL_ID: u8 = 0x6E;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum UtilityMethod {
    AcquireNexUniqueId = 0x1,
}

#[async_trait]
pub trait UtilityProtocol<ClientData: client::ClientData>: Server<ClientData> {
    async fn acquire_nex_unique_id(
        &self,
        client: &mut ClientConnection<ClientData>,
    ) -> Result<Vec<u8>, ResultCode>;

    async fn handle_acquire_nex_unique_id(
        &self,
        client: &mut ClientConnection<ClientData>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.acquire_nex_unique_id(client).await {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }
}
