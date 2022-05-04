mod packet_flag;
mod packet_option;
mod packet_type;
mod result;
mod signature_context;
mod v1;

pub use packet_flag::{PacketFlag, PacketFlags};
pub use packet_option::PacketOption;
pub use packet_type::PacketType;
pub use result::{Error, PacketResult};
pub use signature_context::SignatureContext;
pub use v1::PacketV1;

pub trait Packet {
    const VERSION: u8;

    const CLIENT_ID: u8 = 0xaf;
    const SERVER_ID: u8 = 0xa1;

    fn to_bytes(&self, context: &SignatureContext) -> Vec<u8>;

    fn get_source(&self) -> u8;
    fn set_source(&mut self, value: u8);

    fn get_destination(&self) -> u8;
    fn set_destination(&mut self, value: u8);

    fn get_packet_type(&self) -> PacketType;
    fn set_packet_type(&mut self, value: PacketType);

    fn get_flags(&self) -> PacketFlags;
    fn set_flags(&mut self, value: PacketFlags);

    fn get_session_id(&self) -> u8;
    fn set_session_id(&mut self, value: u8);

    fn get_signature(&self) -> &[u8];
    fn set_signature(&mut self, value: Vec<u8>);

    fn get_sequence_id(&self) -> u16;
    fn set_sequence_id(&mut self, value: u16);

    fn get_connection_signature(&self) -> &[u8];
    fn set_connection_signature(&mut self, value: Vec<u8>);

    fn get_fragment_id(&self) -> u8;
    fn set_fragment_id(&mut self, value: u8);

    fn get_payload(&self) -> &[u8];
    fn set_payload(&mut self, value: Vec<u8>);
}
