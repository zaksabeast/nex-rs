use nex_rs::{macros::NexProtocol, route::NexProtocol};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const USUM_117_PROTOCOL_ID: u8 = 0x75;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum USUM117Method {
    #[protocol_method(output = "u32")]
    Unknown1 = 0x1,
    #[protocol_method(output = "u32")]
    Unknown7 = 0x7,
    #[protocol_method(output = "u32")]
    Unknown9 = 0x9,
    #[protocol_method(output = "u32")]
    Unknown10 = 0xA,
    #[protocol_method(output = "u32")]
    Unknown15 = 0xF,
}

impl NexProtocol for USUM117Method {
    const PROTOCOL_ID: u8 = USUM_117_PROTOCOL_ID;
}
