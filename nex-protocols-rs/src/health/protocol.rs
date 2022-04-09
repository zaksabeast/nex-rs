use async_trait::async_trait;
use nex_rs::client::ClientConnection;
use nex_rs::nex_types::ResultCode;
use nex_rs::packet::{Packet, PacketV1};
use nex_rs::server::Server;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum HealthMethod {
    PingDaemon = 0x1,
    PingDatabase = 0x2,
    RunSanityCheck = 0x3,
    FixSanityErrors = 0x4,
}

#[async_trait(? Send)]
pub trait HealthProtocol: Server {
    async fn ping_daemon(&self, client: &mut ClientConnection) -> Result<Vec<u8>, ResultCode>;
    async fn ping_database(&self, client: &mut ClientConnection) -> Result<Vec<u8>, ResultCode>;
    async fn run_sanity_check(&self, client: &mut ClientConnection) -> Result<Vec<u8>, ResultCode>;
    async fn fix_sanity_errors(&self, client: &mut ClientConnection)
        -> Result<Vec<u8>, ResultCode>;

    async fn handle_ping_daemon(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let rmc_request = packet.get_rmc_request();
        match self.ping_daemon(client).await {
            Ok(data) => {
                self.send_success(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_ping_database(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let rmc_request = packet.get_rmc_request();
        match self.ping_database(client).await {
            Ok(data) => {
                self.send_success(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_run_sanity_check(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let rmc_request = packet.get_rmc_request();
        match self.run_sanity_check(client).await {
            Ok(data) => {
                self.send_success(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_fix_sanity_errors(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let rmc_request = packet.get_rmc_request();
        match self.fix_sanity_errors(client).await {
            Ok(data) => {
                self.send_success(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    rmc_request.protocol_id,
                    rmc_request.method_id,
                    rmc_request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }
}
