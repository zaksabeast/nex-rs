use nex_rs::route::NexProtocol;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum UtilityMethod {
    AcquireNexUniqueId = 0x1,
}

impl NexProtocol for UtilityMethod {
    const PROTOCOL_ID: u8 = 0x6E;
}
