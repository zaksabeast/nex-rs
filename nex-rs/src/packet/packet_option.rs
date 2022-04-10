use super::Error;
use num_enum::IntoPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive)]
#[repr(u8)]
pub enum PacketOption {
    SupportedFunctions = 0,
    ConnectionSignature = 1,
    FragmentId = 2,
    InitialSequenceId = 3,
    MaxSubstreamId = 4,
}

impl TryFrom<u8> for PacketOption {
    type Error = Error;

    fn try_from(raw: u8) -> Result<Self, Self::Error> {
        match raw {
            0x0 => Ok(Self::SupportedFunctions),
            0x1 => Ok(Self::ConnectionSignature),
            0x2 => Ok(Self::FragmentId),
            0x3 => Ok(Self::InitialSequenceId),
            0x4 => Ok(Self::MaxSubstreamId),
            _ => Err(Error::InvalidPacketOption { option: raw }),
        }
    }
}
