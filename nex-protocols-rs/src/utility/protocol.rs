use nex_rs::macros::NexProtocol;
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const UTILITY_PROTOCOL_ID: u8 = 0x6E;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum UtilityMethod {
    #[protocol_method(output = "u64")]
    AcquireNexUniqueId = 0x1,
}
