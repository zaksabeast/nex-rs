use async_trait::async_trait;
use nex_rs::{
    client::ClientConnection, nex_types::ResultCode, result::NexResult, rmc::RMCRequest,
    server::Server,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::sync::Arc;
use tokio::sync::RwLock;

pub const USUM_117_PROTOCOL_ID: u8 = 0x75;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum USUM117Method {
    Unknown1 = 0x1,
    Unknown7 = 0x7,
    Unknown9 = 0x9,
    Unknown10 = 0xA,
    Unknown15 = 0xF,
}

#[async_trait]
pub trait USUM117Protocol: Server {
    async fn unknown_1(&self, client: Arc<RwLock<ClientConnection>>)
        -> Result<Vec<u8>, ResultCode>;
    async fn unknown_7(&self, client: Arc<RwLock<ClientConnection>>)
        -> Result<Vec<u8>, ResultCode>;
    async fn unknown_9(&self, client: Arc<RwLock<ClientConnection>>)
        -> Result<Vec<u8>, ResultCode>;
    async fn unknown_10(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn unknown_15(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;

    async fn handle_unknown_1(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.unknown_1(Arc::clone(&client)).await {
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

    async fn handle_unknown_7(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.unknown_7(Arc::clone(&client)).await {
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

    async fn handle_unknown_9(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.unknown_9(Arc::clone(&client)).await {
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

    async fn handle_unknown_10(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.unknown_10(Arc::clone(&client)).await {
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

    async fn handle_unknown_15(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.unknown_15(Arc::clone(&client)).await {
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
