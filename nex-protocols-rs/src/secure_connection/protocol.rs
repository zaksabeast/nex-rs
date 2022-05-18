use nex_rs::{
    nex_types::{DataHolder, NexList, NexQBuffer, NexString, ResultCode},
    route::NexProtocol,
};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum SecureConnectionMethod {
    Register = 0x1,
    RequestConnectionData = 0x2,
    RequestUrls = 0x3,
    RegisterEx = 0x4,
    TestConnectivity = 0x5,
    UpdateUrls = 0x6,
    ReplaceUrl = 0x7,
    SendReport = 0x8,
}

impl NexProtocol for SecureConnectionMethod {
    const PROTOCOL_ID: u8 = 0xB;
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
