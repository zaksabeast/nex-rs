use super::Error;
use num_enum::IntoPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
#[repr(u16)]
pub enum PacketType {
    Syn = 0x0,
    Connect = 0x1,
    Data = 0x2,
    Disconnect = 0x3,
    Ping = 0x4,
}

impl TryFrom<u16> for PacketType {
    type Error = Error;

    fn try_from(raw: u16) -> Result<Self, Self::Error> {
        match raw {
            0x0 => Ok(Self::Syn),
            0x1 => Ok(Self::Connect),
            0x2 => Ok(Self::Data),
            0x3 => Ok(Self::Disconnect),
            0x4 => Ok(Self::Ping),
            _ => Err(Error::InvalidPacketType { packet_type: raw }),
        }
    }
}
