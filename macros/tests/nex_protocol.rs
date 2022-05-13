use macros::NexProtocol;
use nex_rs::{
    client::{ClientConnection, ClientContext},
    nex_types::ResultCode,
    packet::PacketV1,
    result::NexError,
    rmc::RMCRequest,
    server::{BaseServer, EventHandler, Server, ServerResult},
};
use no_std_io::{EndianRead, EndianWrite, Writer};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;

#[derive(Debug, Default, EndianRead, EndianWrite)]
pub struct AddInput {
    first: u32,
    second: u32,
}

#[derive(Debug, Default, PartialEq, EndianRead, EndianWrite)]
pub struct AddOutput {
    sum: u32,
}

// Methods for the pretend nex "Math" protocol
// Typically this method would be used for routing.
// We need it here to test code generation, but otherwise it won't be used.
#[allow(dead_code)]
#[derive(Debug, IntoPrimitive, TryFromPrimitive, NexProtocol)]
#[repr(u32)]
enum MathMethod {
    #[protocol_method(input = "AddInput", output = "AddOutput")]
    Add = 1,
    #[protocol_method]
    Noop = 2,
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

#[derive(Debug)]
enum Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl NexError for Error {
    fn error_code(&self) -> ResultCode {
        0.into()
    }
}

#[async_trait::async_trait]
impl MathProtocol for MockServer {
    type Error = Error;

    async fn add(
        &self,
        _client: &mut ClientConnection,
        input: AddInput,
    ) -> Result<AddOutput, Error> {
        Ok(AddOutput {
            sum: input.first + input.second,
        })
    }

    async fn noop(&self, _client: &mut ClientConnection) -> Result<(), Error> {
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
async fn generates_method() {
    let (server, mut client) = get_server_and_client();
    let input = AddInput {
        first: 1,
        second: 2,
    };
    let result = server.add(&mut client, input).await.unwrap();
    assert_eq!(result, AddOutput { sum: 3 })
}

#[tokio::test]
async fn generates_handle_method() {
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

    let result = server.handle_add(&mut client, &request).await;
    assert_eq!(result, Ok(()))
}
