use nex_rs::macros::NexProtocol;
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub type Placeholder = u8;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum HealthMethod {
    #[protocol_method(output = "PingDaemonOutput")]
    PingDaemon = 0x1,
    #[protocol_method(output = "PingDatabaseOutput")]
    PingDatabase = 0x2,
    #[protocol_method(output = "RunSanityCheckOutput")]
    RunSanityCheck = 0x3,
    #[protocol_method(output = "FixSanityErrorsOutput")]
    FixSanityErrors = 0x4,
}

#[derive(EndianRead, EndianWrite)]
pub struct PingDaemonOutput {
    pub result: bool,
}

#[derive(EndianRead, EndianWrite)]
pub struct PingDatabaseOutput {
    pub result: bool,
}

#[derive(EndianRead, EndianWrite)]
pub struct RunSanityCheckOutput {
    pub result: bool,
}

#[derive(EndianRead, EndianWrite)]
pub struct FixSanityErrorsOutput {
    pub result: bool,
}
