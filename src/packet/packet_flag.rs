use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::ops::{BitAnd, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum PacketFlag {
    Ack = 0x1,
    Reliable = 0x2,
    NeedsAck = 0x4,
    HasSize = 0x8,
    MultiAck = 0x200,
}

impl BitAnd<u16> for PacketFlag {
    type Output = u16;

    fn bitand(self, rhs: u16) -> Self::Output {
        (self as u16) & rhs
    }
}

impl BitOrAssign<PacketFlag> for PacketFlags {
    fn bitor_assign(&mut self, rhs: PacketFlag) {
        self.0 |= u16::from(rhs);
    }
}

impl BitAnd<PacketFlag> for u16 {
    type Output = u16;

    fn bitand(self, rhs: PacketFlag) -> Self::Output {
        self & (rhs as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PacketFlags(u16);

impl PacketFlags {
    pub fn new(raw: u16) -> Self {
        Self(raw)
    }

    pub fn ack(&self) -> bool {
        self.0 & PacketFlag::Ack != 0
    }

    pub fn reliable(&self) -> bool {
        self.0 & PacketFlag::Reliable != 0
    }

    pub fn needs_ack(&self) -> bool {
        self.0 & PacketFlag::NeedsAck != 0
    }

    pub fn has_size(&self) -> bool {
        self.0 & PacketFlag::HasSize != 0
    }

    pub fn multi_ack(&self) -> bool {
        self.0 & PacketFlag::MultiAck != 0
    }
}

impl From<PacketFlags> for u16 {
    fn from(flags: PacketFlags) -> Self {
        flags.0
    }
}
