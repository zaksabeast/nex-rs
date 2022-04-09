mod base;
mod packet_flag;
mod packet_option;
mod packet_type;
mod v1;

pub use base::BasePacket;
pub use packet_flag::{PacketFlag, PacketFlags};
pub use packet_option::PacketOption;
pub use packet_type::PacketType;
pub use v1::PacketV1;

use crate::client::ClientContext;
use crate::rmc::RMCRequest;

pub trait Packet {
    const VERSION: u8;

    const CLIENT_ID: u8 = 0xaf;
    const SERVER_ID: u8 = 0xa1;

    fn get_base(&self) -> &BasePacket;
    fn get_mut_base(&mut self) -> &mut BasePacket;
    fn to_bytes(&mut self, context: &mut ClientContext) -> Vec<u8>;

    fn get_data(&self) -> &[u8] {
        &self.get_base().data
    }
    fn set_data(&mut self, value: Vec<u8>) {
        self.get_mut_base().data = value;
    }

    fn get_source(&self) -> u8 {
        self.get_base().source
    }
    fn set_source(&mut self, value: u8) {
        self.get_mut_base().source = value;
    }

    fn get_destination(&self) -> u8 {
        self.get_base().destination
    }
    fn set_destination(&mut self, value: u8) {
        self.get_mut_base().destination = value;
    }

    fn get_packet_type(&self) -> PacketType {
        self.get_base().packet_type
    }
    fn set_packet_type(&mut self, value: PacketType) {
        self.get_mut_base().packet_type = value;
    }

    fn get_flags(&self) -> PacketFlags {
        self.get_base().flags
    }
    fn get_mut_flags(&mut self) -> &mut PacketFlags {
        &mut self.get_mut_base().flags
    }
    fn set_flags(&mut self, value: PacketFlags) {
        self.get_mut_base().flags = value;
    }

    fn get_session_id(&self) -> u8 {
        self.get_base().session_id
    }
    fn set_session_id(&mut self, value: u8) {
        self.get_mut_base().session_id = value;
    }

    fn get_signature(&self) -> &[u8] {
        &self.get_base().signature
    }
    fn set_signature(&mut self, value: Vec<u8>) {
        self.get_mut_base().signature = value;
    }

    fn get_sequence_id(&self) -> u16 {
        self.get_base().sequence_id
    }
    fn set_sequence_id(&mut self, value: u16) {
        self.get_mut_base().sequence_id = value;
    }

    fn get_connection_signature(&self) -> &[u8] {
        &self.get_base().connection_signature
    }
    fn set_connection_signature(&mut self, value: Vec<u8>) {
        self.get_mut_base().connection_signature = value;
    }

    fn get_fragment_id(&self) -> u8 {
        self.get_base().fragment_id
    }
    fn set_fragment_id(&mut self, value: u8) {
        self.get_mut_base().fragment_id = value;
    }

    fn get_payload(&self) -> &[u8] {
        &self.get_base().payload
    }
    fn set_payload(&mut self, value: Vec<u8>) {
        self.get_mut_base().payload = value;
    }

    fn get_rmc_request(&self) -> &RMCRequest {
        &self.get_base().rmc_request
    }
    fn set_rmc_request(&mut self, value: RMCRequest) {
        self.get_mut_base().rmc_request = value;
    }
}
