use nex_rs::route::NexProtocol;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum USUM117Method {
    Unknown1 = 0x1,
    Unknown7 = 0x7,
    Unknown9 = 0x9,
    Unknown10 = 0xA,
    Unknown15 = 0xF,
}

impl NexProtocol for USUM117Method {
    const PROTOCOL_ID: u8 = 0x75;
}
