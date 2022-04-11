use nex_rs::nex_types::NexString;
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
