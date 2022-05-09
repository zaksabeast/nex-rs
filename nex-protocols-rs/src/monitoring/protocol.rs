use nex_rs::{
    macros::NexProtocol,
    nex_types::{NexList, NexString},
};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum MonitoringMethod {
    #[protocol_method(output = "PingDaemonOutput")]
    PingDaemon = 0x1,
    #[protocol_method(output = "GetClusterMembersOutput")]
    GetClusterMembers = 0x2,
}

#[derive(EndianRead, EndianWrite)]
pub struct PingDaemonOutput {
    pub result: bool,
}

#[derive(EndianRead, EndianWrite)]
pub struct GetClusterMembersOutput {
    pub result: NexList<NexString>,
}
