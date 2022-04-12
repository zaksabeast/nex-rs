use async_trait::async_trait;
use nex_rs::{
    client::ClientConnection, nex_types::ResultCode, result::NexResult, rmc::RMCRequest,
    server::Server,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const UTILITY_PROTOCOL_ID: u8 = 0x6E;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum UtilityMethod {
    AcquireNexUniqueId = 0x1,
}

#[async_trait(?Send)]
pub trait UtilityProtocol: Server {
    async fn acquire_nex_unique_id(
        &self,
        client: &mut ClientConnection,
    ) -> Result<Vec<u8>, ResultCode>;

    async fn handle_acquire_nex_unique_id(
        &self,
        client: &mut ClientConnection,
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
