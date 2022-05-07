use crate::client::ClientContext;
use getset::{CopyGetters, Getters, Setters};

#[derive(Debug, Getters, CopyGetters, Setters)]
#[getset(skip)]
pub struct ServerSettings {
    #[getset(set = "pub")]
    pub(super) access_key: String,
    #[getset(set = "pub")]
    pub(super) nex_version: u32,
    #[getset(set = "pub")]
    pub(super) fragment_size: u16,
    pub(super) flags_version: u32,
    #[getset(set = "pub")]
    pub(super) ping_timeout: u32,
    pub(super) checksum_version: u32,
}

impl ServerSettings {
    pub fn create_client_context(&self) -> ClientContext {
        ClientContext::new(self.flags_version, &self.access_key)
    }
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            access_key: "".to_string(),
            nex_version: 0,
            fragment_size: 1300,
            ping_timeout: 5,
            flags_version: 1,
            checksum_version: 1,
        }
    }
}
