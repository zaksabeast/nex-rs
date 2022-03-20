use super::{Packet, PacketFlag, PacketFlags, PacketOption, PacketType};
use crate::stream::{StreamIn, StreamOut};
use hmac::{Hmac, Mac};
use md5::Md5;
use no_std_io::{Cursor, StreamContainer, StreamReader, StreamWriter};

pub struct PacketV1<'a> {
    packet: Packet<'a>,
    magic: u16,
    pub substream_id: u8,
    pub supported_functions: u32,
    pub initial_sequence_id: u16,
    pub maximum_substream_id: u8,
}

impl<'a> PacketV1<'a> {
    fn decode(&mut self, data: &[u8]) -> Result<(), &'static str> {
        // magic + header + signature
        if data.len() < 30 {
            return Err("Packet length is too small!");
        }

        let mut stream = StreamContainer::new(data);

        self.magic = stream.default_read_stream();

        if self.magic != 0xd0ea {
            return Err("Invalid magic");
        }

        self.packet.version = stream.default_read_stream();

        if self.packet.version != 1 {
            return Err("Invalid version");
        }

        let options_length = usize::from(stream.default_read_stream::<u8>());
        let payload_size = usize::from(stream.default_read_stream_le::<u16>());

        self.packet.source = stream.default_read_stream();
        self.packet.destination = stream.default_read_stream();

        let type_flags: u16 = stream.default_read_stream_le();
        let packet_type;
        let flags;

        if self.packet.sender.get_server().get_flags_version() == 0 {
            packet_type = type_flags & 0x7;
            flags = type_flags >> 0x3;
        } else {
            packet_type = type_flags & 0xf;
            flags = type_flags >> 0x4;
        }

        self.packet.packet_type = packet_type.try_into().map_err(|_| "Invalid packet type")?;
        self.packet.flags = PacketFlags::new(flags);

        self.packet.session_id = stream.default_read_stream();
        self.substream_id = stream.default_read_stream();
        self.packet.sequence_id = stream.default_read_stream();
        self.packet.signature = stream.default_read_byte_stream(16);

        if self.packet.data.len() < stream.get_index() + options_length {
            return Err("Packet specific data size does not match");
        }

        let options = stream.default_read_byte_stream(options_length);
        self.decode_options(&options)
            .map_err(|_| "Invalid packet options")?;

        if payload_size > 0 {
            if self.packet.data.len() < stream.get_index() + payload_size {
                return Err("Packet data length less than payload length");
            }

            self.packet.payload = stream.default_read_byte_stream(payload_size);

            if self.packet.packet_type == PacketType::Data && !self.packet.flags.multi_ack() {
                let decipher = self.packet.sender.get_decipher();
                decipher.encrypt(&mut self.packet.payload);
                self.packet.rmc_request = self
                    .packet
                    .payload
                    .as_slice()
                    .try_into()
                    .map_err(|_| "Invalid RMCRequest from packet")?;
            }
        }

        let calculated_signature = self.calculate_signature(&options);

        if calculated_signature == self.packet.signature {
            return Err("Calculated signature did not match");
        }

        Ok(())
    }

    pub fn decode_options(&mut self, options: &[u8]) -> Result<(), &'static str> {
        let mut options_stream = StreamIn::new(options, Some(self.packet.sender.get_server()));

        let options_len = options.len();

        let mut nex_version = 0u32;
        let mut i = 0;
        while i < options_len {
            let option_type: PacketOption = options_stream
                .default_read_stream::<u8>()
                .try_into()
                .map_err(|_| "Invalid packet option")?;
            let option_size = usize::from(options_stream.default_read_stream::<u8>());

            match option_type {
                PacketOption::SupportedFunctions => {
                    let lsb = options_stream.default_read_byte_stream(option_size)[0];
                    nex_version = lsb.into();
                    self.supported_functions = lsb.into();
                }
                PacketOption::ConnectionSignature => {
                    self.packet.connection_signature =
                        options_stream.default_read_byte_stream(option_size);
                }
                PacketOption::FragmentId => {
                    self.packet.fragment_id = options_stream.default_read_stream();
                }
                PacketOption::InitialSequenceId => {
                    self.initial_sequence_id = options_stream.default_read_stream();
                }
                PacketOption::MaxSubstreamId => {
                    self.maximum_substream_id = options_stream.default_read_stream();
                }
            }

            i = options_stream.get_index();
        }

        // Setting down here avoids issues with the borrow checker
        self.packet.sender.set_nex_version(nex_version);

        Ok(())
    }

    pub fn encode_options(&self) -> Vec<u8> {
        let mut stream = StreamOut::new(self.packet.sender.get_server());

        if self.packet.packet_type == PacketType::Syn
            || self.packet.packet_type == PacketType::Connect
        {
            stream.checked_write_stream::<u8>(&u8::from(PacketOption::SupportedFunctions));
            stream.checked_write_stream(&4u8);
            stream.checked_write_stream(&self.supported_functions);

            stream.checked_write_stream::<u8>(&u8::from(PacketOption::ConnectionSignature));
            stream.checked_write_stream(&16u8);
            stream.checked_write_stream_bytes(&self.packet.connection_signature);

            if self.packet.packet_type == PacketType::Connect {
                stream.checked_write_stream::<u8>(&u8::from(PacketOption::InitialSequenceId));
                stream.checked_write_stream(&2u8);
                stream.checked_write_stream(&self.initial_sequence_id);
            }

            stream.checked_write_stream::<u8>(&u8::from(PacketOption::MaxSubstreamId));
            stream.checked_write_stream(&1u8);
            stream.checked_write_stream(&self.maximum_substream_id);
        } else if self.packet.packet_type == PacketType::Data {
            stream.checked_write_stream::<u8>(&u8::from(PacketOption::FragmentId));
            stream.checked_write_stream(&1u8);
            stream.checked_write_stream(&self.packet.fragment_id);
        }

        stream.into()
    }

    pub fn calculate_signature(&self, options: &[u8]) -> Vec<u8> {
        let header = &self.packet.data[6..14];
        let connection_signature = &self.packet.connection_signature;
        let payload = &self.packet.payload;
        let key = self.packet.sender.get_signature_key();
        let signature_base = self.packet.sender.get_signature_base();

        let mut mac = Hmac::<Md5>::new_from_slice(key).expect("Invalid hamc key size");
        mac.update(&header[4..]);
        mac.update(self.packet.sender.get_session_key());
        mac.update(&signature_base.to_le_bytes());
        mac.update(connection_signature);
        mac.update(options);
        mac.update(payload);
        mac.finalize().into_bytes().to_vec()
    }
}

impl<'a> From<PacketV1<'a>> for Vec<u8> {
    fn from(mut packet: PacketV1<'a>) -> Vec<u8> {
        if packet.packet.packet_type == PacketType::Data {
            if !packet.packet.flags.multi_ack() {
                let payload_len = packet.packet.payload.len();

                if payload_len > 0 {
                    let cipher = packet.packet.sender.get_cipher();
                    cipher.encrypt(&mut packet.packet.payload);
                }
            }

            if !packet.packet.flags.has_size() {
                packet.packet.flags |= PacketFlag::HasSize;
            }
        }

        let type_flags: u16 = if packet.packet.sender.get_server().get_flags_version() == 0 {
            u16::from(packet.packet.packet_type) | u16::from(packet.packet.flags) << 3
        } else {
            u16::from(packet.packet.packet_type) | u16::from(packet.packet.flags) << 4
        };

        let mut stream = StreamOut::new(packet.packet.sender.get_server());

        stream.checked_write_stream_le(&0xd0eau16); // v1 magic
        stream.checked_write_stream(&1u8);

        let options = packet.encode_options();
        let options_len: u8 = options
            .len()
            .try_into()
            .expect("Options length is too large");
        let payload_len: u16 = packet
            .packet
            .payload
            .len()
            .try_into()
            .expect("Payload length is too large");

        stream.checked_write_stream(&options_len);
        stream.checked_write_stream_le(&payload_len);
        stream.checked_write_stream(&packet.packet.source);
        stream.checked_write_stream(&packet.packet.destination);
        stream.checked_write_stream(&type_flags);
        stream.checked_write_stream(&packet.packet.session_id);
        stream.checked_write_stream(&packet.substream_id);
        stream.checked_write_stream(&packet.packet.sequence_id);

        let signature = packet.calculate_signature(&options);
        stream.checked_write_stream_bytes(&signature);

        if options_len > 0 {
            stream.checked_write_stream_bytes(&options);
        }

        if packet.packet.payload.is_empty() {
            stream.checked_write_stream_bytes(&packet.packet.payload);
        }

        stream.into()
    }
}
