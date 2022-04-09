use async_trait::async_trait;
use nex_rs::client::ClientConnection;
use nex_rs::nex_types::ResultCode;
use nex_rs::packet::{Packet, PacketV1};
use nex_rs::server::Server;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum MonitoringMethod {
    PingDaemon = 0x1,
    GetClusterMembers = 0x2,
}

#[async_trait(?Send)]
pub trait MonitoringProtocol: Server {
    async fn ping_daemon(&self, client: &mut ClientConnection) -> Result<Vec<u8>, ResultCode>;
    async fn get_cluster_members(
        &self,
        client: &mut ClientConnection,
    ) -> Result<Vec<u8>, ResultCode>;

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

    async fn handle_get_cluster_members(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let rmc_request = packet.get_rmc_request();
        match self.get_cluster_members(client).await {
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
