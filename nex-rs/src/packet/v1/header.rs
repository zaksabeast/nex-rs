use super::packet::PacketV1;
use crate::packet::{Packet, PacketFlags, PacketType};
use no_std_io::{EndianRead, EndianWrite, Error, ReadOutput, Reader, Writer};

const HEADER_SIZE: usize = 14;

#[derive(Debug, Clone, Copy)]
pub struct PacketV1Header {
    raw: [u8; HEADER_SIZE],
}

impl PacketV1Header {
    pub fn new(raw: [u8; HEADER_SIZE]) -> Self {
        Self { raw }
    }

    pub fn flags(&self, flags_version: u32) -> PacketFlags {
        let shift = if flags_version == 0 { 3 } else { 4 };
        let flags = self.type_flags() >> shift;
        PacketFlags::new(flags)
    }

    pub fn set_flags(&mut self, flags_version: u32, value: PacketFlags) {
        let shift = if flags_version == 0 { 3 } else { 4 };
        let type_flags = (u16::from(value) << shift) | u16::from(self.packet_type(flags_version));
        self.set_type_flags(type_flags);
    }

    pub fn packet_type(&self, flags_version: u32) -> PacketType {
        let mask = if flags_version == 0 { 0x7 } else { 0xf };
        let packet_type = self.type_flags() & mask;
        packet_type.into()
    }

    pub fn set_packet_type(&mut self, flags_version: u32, value: PacketType) {
        let mask = if flags_version == 0 { !0x7 } else { !0xf };
        let type_flags = (self.type_flags() & mask) | u16::from(value);
        self.set_type_flags(type_flags);
    }

    pub fn magic(&self) -> u16 {
        self.raw.default_read_le(0)
    }
    pub fn set_magic(&mut self, value: u16) {
        self.raw.checked_write_le(0, &value);
    }

    pub fn version(&self) -> u8 {
        self.raw.default_read_le(2)
    }
    pub fn set_version(&mut self, value: u8) {
        self.raw.checked_write_le(2, &value);
    }

    pub fn options_length(&self) -> u8 {
        self.raw.default_read_le(3)
    }
    pub fn set_options_length(&mut self, value: u8) {
        self.raw.checked_write_le(3, &value);
    }

    pub fn payload_size(&self) -> u16 {
        self.raw.default_read_le(4)
    }
    pub fn set_payload_size(&mut self, value: u16) {
        self.raw.checked_write_le(4, &value);
    }

    pub fn source(&self) -> u8 {
        self.raw.default_read_le(6)
    }
    pub fn set_source(&mut self, value: u8) {
        self.raw.checked_write_le(6, &value);
    }

    pub fn destination(&self) -> u8 {
        self.raw.default_read_le(7)
    }
    pub fn set_destination(&mut self, value: u8) {
        self.raw.checked_write_le(7, &value);
    }

    pub fn type_flags(&self) -> u16 {
        self.raw.default_read_le(8)
    }
    pub fn set_type_flags(&mut self, value: u16) {
        self.raw.checked_write_le(8, &value);
    }

    pub fn session_id(&self) -> u8 {
        self.raw.default_read_le(10)
    }
    pub fn set_session_id(&mut self, value: u8) {
        self.raw.checked_write_le(10, &value);
    }

    pub fn substream_id(&self) -> u8 {
        self.raw.default_read_le(11)
    }
    pub fn set_substream_id(&mut self, value: u8) {
        self.raw.checked_write_le(11, &value);
    }

    pub fn sequence_id(&self) -> u16 {
        self.raw.default_read_le(12)
    }
    pub fn set_sequence_id(&mut self, value: u16) {
        self.raw.checked_write_le(12, &value);
    }
}

impl Default for PacketV1Header {
    fn default() -> Self {
        let mut result = Self {
            raw: [0; HEADER_SIZE],
        };
        result.set_magic(0xd0ea);
        result.set_version(PacketV1::VERSION);
        result.set_options_length(0);
        result.set_payload_size(0);
        result.set_source(PacketV1::SERVER_ID);
        result.set_destination(PacketV1::CLIENT_ID);
        result.set_packet_type(0, PacketType::Syn);
        result.set_flags(0, PacketFlags::new(0));
        result.set_session_id(0);
        result.set_substream_id(0);
        result.set_sequence_id(0);
        result
    }
}

impl EndianRead for PacketV1Header {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let raw = bytes.read_byte_vec(0, HEADER_SIZE)?.try_into().unwrap();
        let result = Self::new(raw);
        Ok(ReadOutput::new(result, HEADER_SIZE))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for PacketV1Header {
    fn get_size(&self) -> usize {
        HEADER_SIZE
    }

    fn try_write_le(&self, mut dst: &mut [u8]) -> Result<usize, Error> {
        dst.write_bytes(0, &self.raw)?;
        Ok(HEADER_SIZE)
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
