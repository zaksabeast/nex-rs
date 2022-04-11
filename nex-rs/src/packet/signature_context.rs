use crate::crypto::md5;
use getset::{CopyGetters, Getters, Setters};

#[derive(Clone, Default, CopyGetters, Getters, Setters)]
#[getset(skip)]
pub struct SignatureContext {
    #[getset(get = "pub", set = "pub")]
    server_connection_signature: Vec<u8>,
    #[getset(get = "pub", set = "pub")]
    client_connection_signature: Vec<u8>,
    #[getset(get = "pub", set = "pub")]
    session_key: Vec<u8>,
    #[getset(get = "pub")]
    signature_key: [u8; 16],
    #[getset(get_copy = "pub")]
    signature_base: u32,
}

impl SignatureContext {
    pub fn new(access_key: &str) -> Self {
        Self {
            session_key: vec![],
            server_connection_signature: vec![],
            client_connection_signature: vec![],
            signature_key: md5::hash(access_key.as_bytes()),
            signature_base: access_key.as_bytes().iter().map(|byte| *byte as u32).sum(),
        }
    }
}
