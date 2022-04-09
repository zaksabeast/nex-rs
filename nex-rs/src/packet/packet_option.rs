use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PacketOption {
    SupportedFunctions = 0,
    ConnectionSignature = 1,
    FragmentId = 2,
    InitialSequenceId = 3,
    MaxSubstreamId = 4,
}
