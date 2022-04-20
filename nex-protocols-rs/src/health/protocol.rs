use async_trait::async_trait;
use nex_rs::{
    client::ClientConnection, nex_types::ResultCode, result::NexResult, rmc::RMCRequest,
    server::Server,
};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum HealthMethod {
    PingDaemon = 0x1,
    PingDatabase = 0x2,
    RunSanityCheck = 0x3,
    FixSanityErrors = 0x4,
}

#[async_trait]
pub trait HealthProtocol: Server {
    async fn ping_daemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn ping_database(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn run_sanity_check(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn fix_sanity_errors(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;

    async fn handle_ping_daemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.ping_daemon(Arc::clone(&client)).await {
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

    async fn handle_ping_database(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.ping_database(Arc::clone(&client)).await {
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

    async fn handle_run_sanity_check(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.run_sanity_check(Arc::clone(&client)).await {
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

    async fn handle_fix_sanity_errors(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.fix_sanity_errors(Arc::clone(&client)).await {
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
