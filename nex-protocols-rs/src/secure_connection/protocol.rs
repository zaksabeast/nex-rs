use async_trait::async_trait;
use nex_rs::client::ClientConnection;
use nex_rs::nex_types::{DataHolder, NexList, NexQBuffer, NexString, ResultCode};
use nex_rs::rmc::RMCRequest;
use nex_rs::server::Server;
use no_std_io::{StreamContainer, StreamReader};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SECURE_CONNECTION_PROTOCOL_ID: u8 = 0xB;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum SecureConnectionMethod {
    Register = 0x1,
    RequestConnectionData = 0x2,
    RequestURLs = 0x3,
    RegisterEx = 0x4,
    TestConnectivity = 0x5,
    UpdateURLs = 0x6,
    ReplaceURL = 0x7,
    SendReport = 0x8,
}

#[async_trait(?Send)]
pub trait SecureConnectionProtocol: Server {
    async fn register(
        &self,
        client: &mut ClientConnection,
        my_urls: NexList<NexString>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn request_connection_data(
        &self,
        client: &mut ClientConnection,
        cid_target: u32,
        pid_target: u32,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn request_urls(
        &self,
        client: &mut ClientConnection,
        cid_target: u32,
        pid_target: u32,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn register_ex(
        &self,
        client: &mut ClientConnection,
        my_urls: NexList<NexString>,
        custom_data: DataHolder<NexString>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn test_connectivity(&self, client: &mut ClientConnection) -> Result<(), &'static str>;
    async fn update_urls(
        &self,
        client: &mut ClientConnection,
        my_urls: NexList<NexString>,
    ) -> Result<(), &'static str>;
    async fn replace_url(
        &self,
        client: &mut ClientConnection,
        target: NexString,
        url: NexString,
    ) -> Result<(), &'static str>;
    async fn send_report(
        &self,
        client: &mut ClientConnection,
        report_id: u32,
        report_data: NexQBuffer,
    ) -> Result<(), &'static str>;

    async fn handle_register(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let my_urls = parameters_stream
            .read_stream_le::<NexList<NexString>>()
            .map_err(|_| "Can not read my urls")?;

        match self.register(client, my_urls).await {
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

    async fn handle_request_connection_data(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let cid_target = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read cid target")?;

        let pid_target = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read pid target")?;

        match self
            .request_connection_data(client, cid_target, pid_target)
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

    async fn handle_request_urls(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let cid_target = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read cid target")?;

        let pid_target = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read pid target")?;

        match self.request_urls(client, cid_target, pid_target).await {
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

    async fn handle_register_ex(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let my_urls = parameters_stream
            .read_stream_le::<NexList<NexString>>()
            .map_err(|_| "Can not read my urls")?;

        let custom_data = parameters_stream
            .read_stream_le::<DataHolder<NexString>>()
            .map_err(|_| "Can not read custom data")?;

        match self.register_ex(client, my_urls, custom_data).await {
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

    async fn handle_test_connectivity(
        &self,
        client: &mut ClientConnection,
        _request: &RMCRequest,
    ) -> Result<(), &'static str> {
        self.test_connectivity(client).await
    }

    async fn handle_update_urls(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let my_urls = parameters_stream
            .read_stream_le::<NexList<NexString>>()
            .map_err(|_| "Can not read my urls")?;

        self.update_urls(client, my_urls).await
    }

    async fn handle_replace_url(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let target = parameters_stream
            .read_stream_le::<NexString>()
            .map_err(|_| "Can not read my urls")?;

        let url = parameters_stream
            .read_stream_le::<NexString>()
            .map_err(|_| "Can not read my urls")?;

        self.replace_url(client, target, url).await
    }

    async fn handle_send_report(
        &self,
        client: &mut ClientConnection,
        request: &RMCRequest,
    ) -> Result<(), &'static str> {
        let parameters = request.parameters.as_slice();
        let mut parameters_stream = StreamContainer::new(parameters);

        let report_id = parameters_stream
            .read_stream_le::<u32>()
            .map_err(|_| "Can not read my urls")?;

        let report_data = parameters_stream
            .read_stream_le::<NexQBuffer>()
            .map_err(|_| "Can not read my urls")?;

        self.send_report(client, report_id, report_data).await
    }
}
