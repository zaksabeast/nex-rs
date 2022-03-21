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

pub trait Packet {
    fn get_base(&self) -> &BasePacket;
    fn get_mut_base(&mut self) -> &mut BasePacket;
    fn into_bytes(self, context: &mut ClientContext) -> Vec<u8>;
}
