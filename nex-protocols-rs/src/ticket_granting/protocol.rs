use nex_rs::{
    nex_types::{DataHolder, DateTime, NexBuffer, NexList, NexString, NexStruct, ResultCode},
    route::NexProtocol,
};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum TicketGrantingMethod {
    Login = 0x1,
    LoginEx = 0x2,
    RequestTicket = 0x3,
    GetPID = 0x4,
    GetName = 0x5,
    LoginWithContext = 0x6,
}

impl NexProtocol for TicketGrantingMethod {
    const PROTOCOL_ID: u8 = 0xA;
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct AuthenticationInfo {
    pub token: NexString,
    pub ngs_version: u32,
    pub token_type: u8,
    pub server_version: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginInput {
    pub username: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginOutput {
    pub result: ResultCode,
    pub pid: u32,
    pub kerberos_ticket: NexBuffer,
    pub connection_data: RVConnectionData,
    pub branch: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginExInput {
    pub username: NexString,
    pub auth_info: DataHolder<AuthenticationInfo>,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginExOutput {
    pub result: ResultCode,
    pub pid: u32,
    pub kerberos_ticket: NexBuffer,
    pub connection_data: NexStruct<RVConnectionData>,
    pub branch: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct RequestTicketInput {
    pub user_pid: u32,
    pub server_pid: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct RequestTicketOutput {
    pub result: ResultCode,
    pub kerberos_ticket: NexBuffer,
}

#[derive(EndianRead, EndianWrite)]
pub struct GetPIDInput {
    pub username: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct GetPIDOutput {
    pub id: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct GetNameInput {
    pub id: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct GetNameOutput {
    pub name: NexString,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginWithContextInput {
    pub login_data: DataHolder<AuthenticationInfo>,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginWithContextOutput {
    pub result: ResultCode,
    pub pid: u32,
    pub kerberos_ticket: NexBuffer,
    pub connection_data: RVConnectionData,
}

#[derive(EndianRead, EndianWrite)]
pub struct RVConnectionData {
    pub regular_protocols_url: NexString,
    pub special_protocols_list: NexList<u8>,
    pub special_protocols_url: NexString,
    pub current_time: DateTime,
}

#[derive(EndianRead, EndianWrite)]
pub struct LoginData {
    pub principal_type: i8,
    pub username: NexString,
    pub context: u64,
    pub similar_connection: u32,
}
