use async_trait::async_trait;
use nex_rs::client::ClientConnection;
use nex_rs::nex_types::{DataHolder, NexString, ResultCode};
use nex_rs::result::NexResult;
use nex_rs::rmc::RMCRequest;
use nex_rs::server::Server;
use no_std_io::{EndianRead, EndianWrite, StreamContainer, StreamReader};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const AUTHENTICATION_PROTOCOL_ID: u8 = 0xA;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum TicketGrantingMethod {
    Login = 0x1,
    LoginEx = 0x2,
    RequestTicket = 0x3,
    GetPID = 0x4,
    GetName = 0x5,
    LoginWithParam = 0x6,
}

#[derive(Debug, Default, EndianRead, EndianWrite)]
pub struct AuthenticationInfo {
    token: NexString,
    ngs_version: u32,
    token_type: u8,
    server_version: u32,
}

impl AuthenticationInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait(?Send)]
pub trait TicketGrantingProtocol: Server {
    async fn login(
        &self,
        client: &mut ClientConnection,
        username: String,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn login_ex(
        &self,
        client: &mut ClientConnection,
        username: String,
        ticket_granting_info: AuthenticationInfo,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn request_ticket(
        &self,
        client: &mut ClientConnection,
        user_pid: u32,
        server_pid: u32,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn get_pid(
        &self,
        client: &mut ClientConnection,
        username: String,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn get_name(
        &self,
        client: &mut ClientConnection,
        user_pid: u32,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn login_with_param(&self, client: &mut ClientConnection) -> Result<Vec<u8>, ResultCode>;

    async fn handle_login(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let username: String = parameters_stream
            .read_stream_le::<NexString>()
            .map_err(|_| "Can not read username")?
            .into();

        if username.trim().is_empty() {
            return Err("Failed to read username".into());
        }

        match self.login(client, username).await {
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

    async fn handle_login_ex(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let username: String = parameters_stream
            .read_stream_le::<NexString>()
            .map_err(|_| "Can not read username")?
            .into();

        if username.trim().is_empty() {
            return Err("Failed to read username".into());
        }

        let data_holder = parameters_stream
            .read_stream_le::<DataHolder<AuthenticationInfo>>()
            .map_err(|_| "Can not read data holder")?;

        let data_holder_name: String = data_holder.get_name().into();

        if data_holder_name != "AuthenticationInfo" {
            return Err("Data holder name mismatch".into());
        }

        match self
            .login_ex(client, username, data_holder.into_object())
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

    async fn handle_request_ticket(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();
        if parameters.len() != 8 {
            return Err("[TicketGrantingProtocol::request_ticket] Parameters length not 8".into());
        }

        let mut parameters_stream = StreamContainer::new(parameters);

        let user_pid: u32 = parameters_stream
            .read_stream_le()
            .map_err(|_| "[TicketGrantingProtocol::request_ticket] Failed to read user pid")?;
        let server_pid: u32 = parameters_stream
            .read_stream_le()
            .map_err(|_| "[TicketGrantingProtocol::request_ticket] Failed to read server pid")?;

        match self.request_ticket(client, user_pid, server_pid).await {
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

    async fn handle_get_pid(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);
        let username: String = parameters_stream
            .read_stream_le::<NexString>()
            .map_err(|_| "Can not read username")?
            .into();

        if username.trim().is_empty() {
            return Err("[TicketGrantingProtocol::get_pid] Failed to read username".into());
        }

        match self.get_pid(client, username).await {
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

    async fn handle_get_name(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        if parameters.len() != 4 {
            return Err("[TicketGrantingProtocol::get_name] Parameters length not 4".into());
        }

        let mut parameters_stream = StreamContainer::new(parameters);

        let user_pid: u32 = parameters_stream
            .read_stream_le()
            .map_err(|_| "[TicketGrantingProtocol::get_name] Failed to read user PID")?;

        match self.get_name(client, user_pid).await {
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
