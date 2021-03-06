use macros::{nex_method, nex_route};
use nex_rs::{
    client::{ClientConnection, ClientContext},
    nex_types::Empty,
    packet::PacketV1,
    result::SuccessfulResult,
    rmc::RMCRequest,
    route::{NexProtocol, Route},
    server::{BaseServer, EventHandler, Server, ServerResult},
};
use no_std_io::{EndianRead, EndianWrite, Writer};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Default, EndianRead, EndianWrite)]
pub struct AddInput {
    first: u32,
    second: u32,
}

#[derive(Debug, Default, PartialEq, EndianRead, EndianWrite)]
pub struct AddOutput {
    sum: u32,
}

#[derive(Debug, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum MathMethod {
    Add = 1,
    Noop = 2,
}

impl NexProtocol for MathMethod {
    const PROTOCOL_ID: u8 = 1;
}

#[nex_method(method = MathMethod::Add)]
async fn add(
    _server: &MockServer,
    _client: &ClientConnection,
    input: AddInput,
) -> SuccessfulResult<AddOutput> {
    Ok(AddOutput {
        sum: input.first + input.second,
    })
}

#[nex_method(method = MathMethod::Noop)]
async fn noop(_server: &MockServer, _client: &ClientConnection) -> SuccessfulResult<Empty> {
    Ok(Empty)
}

#[derive(Default)]
struct MockServer {
    base: BaseServer,
}

#[async_trait::async_trait]
impl EventHandler for MockServer {
    async fn on_syn(&self, _client: &mut ClientConnection, _packet: &PacketV1) -> ServerResult<()> {
        Ok(())
    }
    async fn on_connect(
        &self,
        _client: &mut ClientConnection,
        _packet: &PacketV1,
    ) -> ServerResult<()> {
        Ok(())
    }
    async fn on_data(
        &self,
        _client: &mut ClientConnection,
        _packet: &PacketV1,
    ) -> ServerResult<()> {
        Ok(())
    }
    async fn on_disconnect(
        &self,
        _client: &mut ClientConnection,
        _packet: &PacketV1,
    ) -> ServerResult<()> {
        Ok(())
    }
    async fn on_ping(
        &self,
        _client: &mut ClientConnection,
        _packet: &PacketV1,
    ) -> ServerResult<()> {
        Ok(())
    }
    async fn on_rmc_request(
        &self,
        _client: &mut ClientConnection,
        _rmc_request: &RMCRequest,
    ) -> ServerResult<()> {
        Ok(())
    }
    async fn on_protocol_method(&self, _method_name: String) {}
    async fn on_error(&self, _error: &nex_rs::result::Error) {}
}

#[async_trait::async_trait]
impl Server for MockServer {
    fn get_base(&self) -> &BaseServer {
        &self.base
    }

    fn get_mut_base(&mut self) -> &mut BaseServer {
        &mut self.base
    }

    async fn send_success<MethodId: Into<u32> + Send, Data: Into<Vec<u8>> + Send>(
        &self,
        _client: &mut ClientConnection,
        _protocol_id: u8,
        _method_id: MethodId,
        _call_id: u32,
        _data: Data,
    ) -> ServerResult<()> {
        // stub
        Ok(())
    }

    async fn send_error<MethodId: Into<u32> + Send>(
        &self,
        _client: &mut ClientConnection,
        _protocol_id: u8,
        _method_id: MethodId,
        _call_id: u32,
        _error_code: u32,
    ) -> ServerResult<()> {
        // stub
        Ok(())
    }
}

fn get_server_and_client() -> (MockServer, ClientConnection) {
    // Set up server
    let server = MockServer::default();

    // Set up client
    let addr = "127.0.0.1:12345".parse().unwrap();
    let context = ClientContext::new(0, "");
    let client = ClientConnection::new(addr, context, 0);

    (server, client)
}

#[tokio::test]
async fn generates_route() {
    let (server, mut client) = get_server_and_client();
    let mut input = vec![];
    input.checked_write_le(0, &AddInput::default());

    let request = RMCRequest {
        protocol_id: 1,
        call_id: 1,
        method_id: 1,
        custom_id: 1,
        parameters: input,
    };

    let result = Route::<{ MathMethod::PROTOCOL_ID as u8 }, { MathMethod::Add as u32 }>::run(
        &server,
        &mut client,
        &request,
    )
    .await;

    assert_eq!(result, Ok(()))
}

#[tokio::test]
async fn routes_method() {
    let (server, mut client) = get_server_and_client();
    let mut input = vec![];
    input.checked_write_le(0, &AddInput::default());

    let request = RMCRequest {
        protocol_id: 1,
        call_id: 1,
        method_id: 1,
        custom_id: 1,
        parameters: input,
    };

    let result = nex_route![MathMethod::Add](&server, &mut client, &request).await;

    assert_eq!(result, Ok(()))
}
