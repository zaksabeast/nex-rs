use crate::matchmake_extension::MatchmakeSessionSearchCriteria;
use async_trait::async_trait;
use nex_rs::client::ClientConnection;
use nex_rs::nex_types::{ResultCode, ResultRange};
use nex_rs::packet::{Packet, PacketV1};
use nex_rs::server::Server;
use no_std_io::{StreamContainer, StreamReader};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const MATCHMAKE_EXTENSION_PROTOCOL_ID: u8 = 0x6D;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum MatchmakeExtensionMethod {
    CloseParticipation = 0x1,
    OpenParticipation = 0x2,
    BrowseMatchmakeSession = 0x4,
    BrowseMatchmakeSessionWithHostUrls = 0x5,
    GetAttractionStatus = 0x31,
    SimpleMatchmake = 0x33,
}

#[async_trait(?Send)]
pub trait MatchmakeExtensionProtocol: Server {
    async fn close_participation(
        &self,
        client: &mut ClientConnection,
        gid: u32,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn open_participation(
        &self,
        client: &mut ClientConnection,
        gid: u32,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn browse_matchmake_session(
        &self,
        client: &mut ClientConnection,
        matchmake_session_search_criteria: MatchmakeSessionSearchCriteria,
        result_range: ResultRange,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn browse_matchmake_session_with_host_urls(
        &self,
        client: &mut ClientConnection,
        matchmake_session_search_criteria: MatchmakeSessionSearchCriteria,
        result_range: ResultRange,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn get_attraction_status(
        &self,
        client: &mut ClientConnection,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn simple_matchmake(
        &self,
        client: &mut ClientConnection,
        group_id: u32,
    ) -> Result<Vec<u8>, ResultCode>;

    async fn handle_close_participation(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let request = packet.get_rmc_request();
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let gid = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read group id")?;

        match self.close_participation(client, gid).await {
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

    async fn handle_open_participation(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let request = packet.get_rmc_request();
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let gid = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read group id")?;

        match self.open_participation(client, gid).await {
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

    async fn handle_browse_matchmake_session(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let request = packet.get_rmc_request();
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let matchmake_session_search_criteria = parameters_stream
            .read_stream_le::<MatchmakeSessionSearchCriteria>()
            .map_err(|_| "Can not read matchmake session search criteria")?;

        let result_range = parameters_stream
            .read_stream_le::<ResultRange>()
            .map_err(|_| "Can not read result range")?;

        match self
            .browse_matchmake_session(client, matchmake_session_search_criteria, result_range)
            .await
        {
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

    async fn handle_browse_matchmake_session_with_host_urls(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let request = packet.get_rmc_request();
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let matchmake_session_search_criteria = parameters_stream
            .read_stream_le::<MatchmakeSessionSearchCriteria>()
            .map_err(|_| "Can not read matchmake session search criteria")?;

        let result_range = parameters_stream
            .read_stream_le::<ResultRange>()
            .map_err(|_| "Can not read result range")?;

        match self
            .browse_matchmake_session_with_host_urls(
                client,
                matchmake_session_search_criteria,
                result_range,
            )
            .await
        {
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

    async fn handle_get_attraction_status(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let request = packet.get_rmc_request();
        match self.get_attraction_status(client).await {
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

    async fn handle_simple_matchmake(
        &self,
        client: &mut ClientConnection,
        packet: &PacketV1,
    ) -> Result<(), &'static str> {
        let request = packet.get_rmc_request();
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let group_id = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read group id")?;

        match self.simple_matchmake(client, group_id).await {
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
