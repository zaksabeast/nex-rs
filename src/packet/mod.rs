mod base;
mod packet_flag;
mod packet_option;
mod packet_type;
mod v1;

use base::BasePacket;
use packet_flag::{PacketFlag, PacketFlags};
use packet_option::PacketOption;
use packet_type::PacketType;

pub trait Packet<'a> {
    fn get_base(&self) -> &BasePacket<'a>;
    fn get_mut_base(&mut self) -> &mut BasePacket<'a>;
}
