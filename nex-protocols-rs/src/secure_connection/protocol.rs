use nex_rs::{
    macros::NexProtocol,
    nex_types::{DataHolder, NexList, NexQBuffer, NexString, ResultCode},
};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const SECURE_CONNECTION_PROTOCOL_ID: u8 = 0xB;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum SecureConnectionMethod {
    #[protocol_method(input = "RegisterInput", output = "RegisterOutput")]
    Register = 0x1,
    #[protocol_method(
        input = "RequestConnectionDataInput",
        output = "RequestConnectionDataOutput"
    )]
    RequestConnectionData = 0x2,
    #[protocol_method(input = "RequestUrlsInput", output = "RequestUrlsOutput")]
    RequestUrls = 0x3,
    #[protocol_method(input = "RegisterExInput", output = "RegisterExOutput")]
    RegisterEx = 0x4,
    #[protocol_method]
    TestConnectivity = 0x5,
    #[protocol_method(input = "UpdateUrlsInput")]
    UpdateUrls = 0x6,
    #[protocol_method(input = "ReplaceUrlInput")]
    ReplaceUrl = 0x7,
    #[protocol_method(input = "SendReportInput")]
    SendReport = 0x8,
}

#[derive(EndianRead, EndianWrite)]
pub struct RegisterInput {
    pub my_urls: NexList<NexString>,
}

#[derive(EndianRead, EndianWrite)]
pub struct RegisterOutput {
    pub result: ResultCode,
    pub pid_connection_id: u32,
    pub public_url: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct RequestConnectionDataInput {
    pub cid_target: u32,
    pub pid_target: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct RequestConnectionDataOutput {
    pub result: bool,
    pub connections_data: u8,
}

#[derive(EndianRead, EndianWrite)]
pub struct RequestUrlsInput {
    pub cid_target: u32,
    pub pid_target: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct RequestUrlsOutput {
    pub result: bool,
    pub urls: NexList<NexString>,
}

#[derive(EndianRead, EndianWrite)]
pub struct RegisterExInput {
    pub my_urls: NexList<NexString>,
    pub custom_data: DataHolder<NexString>,
}

#[derive(EndianRead, EndianWrite)]
pub struct RegisterExOutput {
    pub result: ResultCode,
    pub pid_connection_id: u32,
    pub public_url: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct UpdateUrlsInput {
    pub my_urls: NexList<NexString>,
}

#[derive(EndianRead, EndianWrite)]
pub struct ReplaceUrlInput {
    pub target: NexString,
    pub url: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct SendReportInput {
    pub id: u32,
    pub data: NexQBuffer,
}

#[derive(EndianRead, EndianWrite)]
pub struct ConnectionData {
    pub string: NexString,
    pub connection_id: u32,
}
