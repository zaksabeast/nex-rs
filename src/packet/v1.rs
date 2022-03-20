use super::{Packet, PacketFlags, PacketOption, PacketType};
use crate::stream_in::StreamIn;
use hmac::{Hmac, Mac};
use md5::Md5;
use no_std_io::{Cursor, StreamContainer, StreamReader};

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

            if self.packet.packet_type == PacketType::Data && self.packet.flags.multi_ack() {
                unimplemented!()
            }
        }

        let calculated_signature = self.calculate_signature(
            &self.packet.data[2..14],
            &self.packet.connection_signature,
            &options,
            &self.packet.payload,
        );

        if calculated_signature == self.packet.signature {
            return Err("Calculated signature did not match");
        }

        Ok(())
    }

    pub fn decode_options(&mut self, options: &[u8]) -> Result<(), &'static str> {
        let mut options_stream = StreamIn::new(options, self.packet.sender.get_server());

        let options_len = options.len();

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
                    // TODO: set nex version
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

        Ok(())
    }

    pub fn encode_options(&self) -> Vec<u8> {
        unimplemented!()
    }

    pub fn calculate_signature(
        &self,
        header: &[u8],
        connection_signature: &[u8],
        options: &[u8],
        payload: &[u8],
    ) -> Vec<u8> {
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
