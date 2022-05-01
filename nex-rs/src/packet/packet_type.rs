use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum PacketType {
    Syn = 0x0,
    Connect = 0x1,
    Data = 0x2,
    Disconnect = 0x3,
    Ping = 0x4,
    #[num_enum(default)]
    Invalid = 0xff,
}
