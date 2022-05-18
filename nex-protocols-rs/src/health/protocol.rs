use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum HealthMethod {
    PingDaemon = 0x1,
    PingDatabase = 0x2,
    RunSanityCheck = 0x3,
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
