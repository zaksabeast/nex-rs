use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum PacketFlag {
    Ack = 0x1,
    Reliable = 0x2,
    NeedsAck = 0x4,
    HasSize = 0x8,
    MultiAck = 0x200,
}
