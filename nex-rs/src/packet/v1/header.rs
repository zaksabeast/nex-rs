use super::packet::PacketV1;
use crate::packet::{Error, Packet, PacketFlags, PacketResult, PacketType};
use no_std_io::{EndianRead, EndianWrite};

#[derive(Debug, Default, EndianRead, EndianWrite)]
pub struct RawPacketV1Header {
    pub(super) magic: u16,
    pub(super) version: u8,
    pub(super) options_length: u8,
    pub(super) payload_size: u16,
    pub(super) source: u8,
    pub(super) destination: u8,
    pub(super) type_flags: u16,
    pub(super) session_id: u8,
    pub(super) substream_id: u8,
    pub(super) sequence_id: u16,
}

impl RawPacketV1Header {
    pub fn into_header(&self, flags_version: u32) -> PacketResult<PacketV1Header> {
        if self.magic != 0xd0ea {
            return Err(Error::InvalidMagic { magic: self.magic });
        }

        if self.version != PacketV1::VERSION {
            return Err(Error::InvalidVersion {
                version: self.version,
            });
        }

        let packet_type;
        let flags;

        if flags_version == 0 {
            packet_type = self.type_flags & 0x7;
            flags = self.type_flags >> 0x3;
        } else {
            packet_type = self.type_flags & 0xf;
            flags = self.type_flags >> 0x4;
        }

        let header = PacketV1Header {
            magic: self.magic,
            version: self.version,
            options_length: self.options_length,
            payload_size: self.payload_size,
            source: self.source,
            destination: self.destination,
            session_id: self.session_id,
            substream_id: self.substream_id,
            sequence_id: self.sequence_id,
            flags: PacketFlags::new(flags),
            packet_type: packet_type.try_into()?,
        };

        Ok(header)
    }
}

#[derive(Debug)]
pub struct PacketV1Header {
    pub(super) magic: u16,
    pub(super) version: u8,
    pub(super) options_length: u8,
    pub(super) payload_size: u16,
    pub(super) source: u8,
    pub(super) destination: u8,
    pub(super) packet_type: PacketType,
    pub(super) flags: PacketFlags,
    pub(super) session_id: u8,
    pub(super) substream_id: u8,
    pub(super) sequence_id: u16,
}

impl PacketV1Header {
    pub fn into_raw(&self, flags_version: u32) -> RawPacketV1Header {
        let type_flags: u16 = if flags_version == 0 {
            u16::from(self.packet_type) | u16::from(self.flags) << 3
        } else {
            u16::from(self.packet_type) | u16::from(self.flags) << 4
        };

        RawPacketV1Header {
            type_flags,
            magic: self.magic,
            version: self.version,
            options_length: self.options_length,
            payload_size: self.payload_size,
            source: self.source,
            destination: self.destination,
            session_id: self.session_id,
            substream_id: self.substream_id,
            sequence_id: self.sequence_id,
        }
    }
}

impl Default for PacketV1Header {
    fn default() -> Self {
        Self {
            magic: 0xd0ea,
            version: PacketV1::VERSION,
            options_length: 0,
            payload_size: 0,
            source: PacketV1::SERVER_ID,
            destination: PacketV1::CLIENT_ID,
            packet_type: PacketType::Syn,
            flags: PacketFlags::new(0),
            session_id: 0,
            substream_id: 0,
            sequence_id: 0,
        }
    }
}
