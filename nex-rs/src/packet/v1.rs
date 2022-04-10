use super::{
    BasePacket, Packet, PacketFlag, PacketFlags, PacketOption, PacketType, SignatureContext,
};
use hmac::{Hmac, Mac};
use md5::Md5;
use no_std_io::{Cursor, Reader, StreamContainer, StreamReader, StreamWriter};

#[derive(Debug, Default)]
pub struct PacketV1 {
    base: BasePacket,
    magic: u16,
    substream_id: u8,
    supported_functions: u32,
    initial_sequence_id: u16,
    maximum_substream_id: u8,
}

impl Packet for PacketV1 {
    const VERSION: u8 = 1;

    fn get_base(&self) -> &BasePacket {
        &self.base
    }

    fn get_mut_base(&mut self) -> &mut BasePacket {
        &mut self.base
    }

    fn to_bytes(self: &mut PacketV1, flags_version: u32, context: &SignatureContext) -> Vec<u8> {
        if self.base.packet_type == PacketType::Data && !self.base.flags.has_size() {
            self.base.flags |= PacketFlag::HasSize;
        }

        let type_flags: u16 = if flags_version == 0 {
            u16::from(self.base.packet_type) | u16::from(self.base.flags) << 3
        } else {
            u16::from(self.base.packet_type) | u16::from(self.base.flags) << 4
        };

        let mut stream = StreamContainer::new(vec![]);

        stream.checked_write_stream_le(&0xd0eau16); // v1 magic
        stream.checked_write_stream_le(&1u8);

        let options = self.encode_options();
        let options_len: u8 = options
            .len()
            .try_into()
            .expect("Options length is too large");
        let payload_len: u16 = self
            .base
            .payload
            .len()
            .try_into()
            .expect("Payload length is too large");

        stream.checked_write_stream_le(&options_len);
        stream.checked_write_stream_le(&payload_len);
        stream.checked_write_stream_le(&self.base.source);
        stream.checked_write_stream_le(&self.base.destination);
        stream.checked_write_stream_le(&type_flags);
        stream.checked_write_stream_le(&self.base.session_id);
        stream.checked_write_stream_le(&self.substream_id);
        stream.checked_write_stream_le(&self.base.sequence_id);

        let header = &stream.get_slice()[2..14];
        let signature = Self::calculate_signature(
            header,
            &self.base.payload,
            context.client_connection_signature(),
            &options,
            context,
        )
        .expect("Signature could not be calculated");

        stream.checked_write_stream_bytes(&signature);

        if options_len > 0 {
            stream.checked_write_stream_bytes(&options);
        }

        if !self.base.payload.is_empty() {
            stream.checked_write_stream_bytes(&self.base.payload);
        }

        stream.into_raw()
    }
}

impl PacketV1 {
    pub fn new_ping_packet() -> Self {
        Self {
            base: BasePacket {
                source: Self::SERVER_ID,
                destination: Self::CLIENT_ID,
                packet_type: PacketType::Ping,
                flags: PacketFlag::Ack | PacketFlag::Reliable,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn new_data_packet(
        session_id: u8,
        connection_signature: Vec<u8>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            base: BasePacket {
                payload,
                session_id,
                connection_signature,
                source: Self::SERVER_ID,
                destination: Self::CLIENT_ID,
                flags: PacketFlag::Reliable | PacketFlag::NeedsAck | PacketFlag::HasSize,
                packet_type: PacketType::Data,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn new_ack_packet(&self) -> Self {
        Self {
            base: BasePacket {
                source: self.get_destination(),
                destination: self.get_source(),
                packet_type: self.get_packet_type(),
                sequence_id: self.get_sequence_id(),
                fragment_id: self.get_fragment_id(),
                flags: PacketFlag::Ack | PacketFlag::HasSize,
                ..Default::default()
            },
            substream_id: 0,
            ..Default::default()
        }
    }

    pub fn read_packet(
        data: Vec<u8>,
        flags_version: u32,
        context: &SignatureContext,
    ) -> Result<Self, &'static str> {
        let data_len = data.len();

        let mut packet = Self {
            base: BasePacket::new(data),
            ..Default::default()
        };

        if data_len > 0 {
            packet.decode(flags_version, context)?;
        }

        Ok(packet)
    }

    pub fn get_substream_id(&self) -> u8 {
        self.substream_id
    }
    pub fn set_substream_id(&mut self, value: u8) {
        self.substream_id = value;
    }

    pub fn get_supported_functions(&self) -> u32 {
        self.supported_functions
    }
    pub fn set_supported_functions(&mut self, value: u32) {
        self.supported_functions = value;
    }

    pub fn get_initial_sequence_id(&self) -> u16 {
        self.initial_sequence_id
    }
    pub fn set_initial_sequence_id(&mut self, value: u16) {
        self.initial_sequence_id = value;
    }

    pub fn get_maximum_substream_id(&self) -> u8 {
        self.maximum_substream_id
    }
    pub fn set_maximum_substream_id(&mut self, value: u8) {
        self.maximum_substream_id = value;
    }

    fn decode(
        &mut self,
        flags_version: u32,
        context: &SignatureContext,
    ) -> Result<(), &'static str> {
        let data_len = self.base.data.len();
        let data = self.base.data.clone();

        // magic + header + signature
        if data_len < 30 {
            return Err("Packet length is too small!");
        }

        let mut stream = StreamContainer::new(data.as_slice());

        self.magic = stream.default_read_stream_le();

        if self.magic != 0xd0ea {
            return Err("Invalid magic");
        }

        let version: u8 = stream.default_read_stream_le();
        if version != 1 {
            return Err("Invalid version");
        }

        let options_length = usize::from(stream.default_read_stream_le::<u8>());
        let payload_size = usize::from(stream.default_read_stream_le::<u16>());

        self.base.source = stream.default_read_stream_le();
        self.base.destination = stream.default_read_stream_le();

        let type_flags: u16 = stream.default_read_stream_le();
        let packet_type;
        let flags;

        if flags_version == 0 {
            packet_type = type_flags & 0x7;
            flags = type_flags >> 0x3;
        } else {
            packet_type = type_flags & 0xf;
            flags = type_flags >> 0x4;
        }

        self.base.packet_type = packet_type.try_into().map_err(|_| "Invalid packet type")?;
        self.base.flags = PacketFlags::new(flags);

        self.base.session_id = stream.default_read_stream_le();
        self.substream_id = stream.default_read_stream_le();
        self.base.sequence_id = stream.default_read_stream_le();
        self.base.signature = stream.default_read_byte_stream(16);

        if data_len < stream.get_index() + options_length {
            return Err("Packet specific data size does not match");
        }

        let options = stream.default_read_byte_stream(options_length);

        self.decode_options(&options)?;

        if payload_size > 0 {
            self.base.payload = stream.default_read_byte_stream(payload_size);
        }

        let header = &data[2..14];
        let calculated_signature = Self::calculate_signature(
            header,
            &self.base.payload,
            context.server_connection_signature(),
            &options,
            context,
        )?;

        if calculated_signature != self.base.signature {
            return Err("Calculated signature did not match");
        }

        Ok(())
    }

    pub fn decode_options(&mut self, options: &[u8]) -> Result<(), &'static str> {
        let mut options_stream = StreamContainer::new(options);
        let options_len = options.len();

        let mut i = 0;
        while i < options_len {
            let option_type: PacketOption = options_stream
                .default_read_stream_le::<u8>()
                .try_into()
                .map_err(|_| "Invalid packet option")?;
            let option_size = usize::from(options_stream.default_read_stream_le::<u8>());

            match option_type {
                PacketOption::SupportedFunctions => {
                    let lsb = options_stream.default_read_byte_stream(option_size)[0];
                    // TODO: Set nex version
                    // Is this something we want clients controlling?
                    // Should we know this already?
                    self.supported_functions = lsb.into();
                }
                PacketOption::ConnectionSignature => {
                    self.base.connection_signature =
                        options_stream.default_read_byte_stream(option_size);
                }
                PacketOption::FragmentId => {
                    self.base.fragment_id = options_stream.default_read_stream_le();
                }
                PacketOption::InitialSequenceId => {
                    self.initial_sequence_id = options_stream.default_read_stream_le();
                }
                PacketOption::MaxSubstreamId => {
                    self.maximum_substream_id = options_stream.default_read_stream_le();
                }
            }

            i = options_stream.get_index();
        }

        Ok(())
    }

    pub fn encode_options(&self) -> Vec<u8> {
        let mut stream = StreamContainer::new(vec![]);

        if self.base.packet_type == PacketType::Syn || self.base.packet_type == PacketType::Connect
        {
            stream.checked_write_stream_le::<u8>(&u8::from(PacketOption::SupportedFunctions));
            stream.checked_write_stream_le(&4u8);
            stream.checked_write_stream_le(&self.supported_functions);

            stream.checked_write_stream_le::<u8>(&u8::from(PacketOption::ConnectionSignature));
            stream.checked_write_stream_le(&16u8);
            stream.checked_write_stream_bytes(&self.base.connection_signature);

            if self.base.packet_type == PacketType::Connect {
                stream.checked_write_stream_le::<u8>(&u8::from(PacketOption::InitialSequenceId));
                stream.checked_write_stream_le(&2u8);
                stream.checked_write_stream_le(&self.initial_sequence_id);
            }

            stream.checked_write_stream_le::<u8>(&u8::from(PacketOption::MaxSubstreamId));
            stream.checked_write_stream_le(&1u8);
            stream.checked_write_stream_le(&self.maximum_substream_id);
        } else if self.base.packet_type == PacketType::Data {
            stream.checked_write_stream_le::<u8>(&u8::from(PacketOption::FragmentId));
            stream.checked_write_stream_le(&1u8);
            stream.checked_write_stream_le(&self.base.fragment_id);
        }

        stream.into_raw()
    }

    pub fn calculate_signature(
        header: &[u8],
        payload: &[u8],
        connection_signature: &[u8],
        options: &[u8],
        context: &SignatureContext,
    ) -> Result<Vec<u8>, &'static str> {
        if header.len() < 8 {
            return Err("Header is too small");
        }

        let key = context.signature_key();
        let signature_base = context.signature_base();

        let mut mac = Hmac::<Md5>::new_from_slice(key).map_err(|_| "Invalid hamc key size")?;
        mac.update(&header[4..]);
        mac.update(context.session_key());
        mac.update(&signature_base.to_le_bytes());
        mac.update(connection_signature);
        mac.update(options);
        mac.update(payload);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const BASE_PACKET: [u8; 57] = [
        0xea, 0xd0, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x86,
        0x70, 0xe5, 0x83, 0x0f, 0x94, 0x72, 0x8f, 0x38, 0xc3, 0x29, 0xc6, 0x46, 0x52, 0x7b, 0x1f,
        0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, 0x00,
    ];

    #[test]
    fn should_encode_and_decode() {
        let bytes = BASE_PACKET.to_vec();
        let flags_version = 1;
        let context = SignatureContext::default();
        let mut packet = PacketV1::read_packet(bytes.clone(), flags_version, &context)
            .expect("Should have succeeded!");
        let result = packet.to_bytes(flags_version, &context);
        assert_eq!(result, bytes);
    }

    mod syn {
        use super::*;

        #[test]
        fn should_decode_packet() {
            let bytes = vec![
                0xea, 0xd0, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
                0xd3, 0x7f, 0xf5, 0x70, 0x42, 0x0b, 0xba, 0xbf, 0xa3, 0xb6, 0xc3, 0x47, 0x5e, 0x14,
                0x99, 0x61, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01,
                0x01,
            ];

            let flags_version = 1;
            let context = SignatureContext::default();
            let packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            assert_eq!(packet.base.packet_type, PacketType::Syn);
            assert_eq!(packet.base.flags.needs_ack(), true);
            assert_eq!(packet.base.flags.has_size(), true);
            assert_eq!(packet.supported_functions, 4);
            assert_eq!(packet.maximum_substream_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.base.packet_type = PacketType::Syn;
            packet.base.flags.clear_flags();
            packet.base.flags.set_flag(PacketFlag::NeedsAck);
            packet.base.flags.set_flag(PacketFlag::HasSize);
            packet.supported_functions = 4;
            packet.maximum_substream_id = 1;

            let result: Vec<u8> = packet.to_bytes(flags_version, &context);
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
                0xd3, 0x7f, 0xf5, 0x70, 0x42, 0x0b, 0xba, 0xbf, 0xa3, 0xb6, 0xc3, 0x47, 0x5e, 0x14,
                0x99, 0x61, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01,
                0x01,
            ];
            assert_eq!(result, expected_result);
        }
    }

    mod connect {
        use super::*;

        #[test]
        fn should_decode_packet() {
            let bytes = vec![
                0xea, 0xd0, 0x01, 0x1f, 0x01, 0x00, 0x00, 0x00, 0xe1, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x15, 0xab, 0x64, 0x8a, 0xc2, 0xea, 0xcd, 0xa7, 0x25, 0x20, 0x19, 0x6f, 0x58, 0x0e,
                0xea, 0x14, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x02,
                0xcd, 0xab, 0x04, 0x01, 0x00, 0xaa,
            ];

            let flags_version = 1;
            let context = SignatureContext::default();
            let packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            assert_eq!(packet.base.packet_type, PacketType::Connect);
            assert_eq!(packet.base.flags.reliable(), true);
            assert_eq!(packet.base.flags.needs_ack(), true);
            assert_eq!(packet.base.flags.has_size(), true);
            assert_eq!(packet.supported_functions, 4);
            assert_eq!(packet.maximum_substream_id, 0);
            assert_eq!(packet.initial_sequence_id, 0xabcd);
            assert_eq!(packet.base.payload, vec![0xaa]);
            assert_eq!(packet.base.session_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.base.packet_type = PacketType::Connect;
            packet.base.flags.clear_flags();
            packet.base.flags.set_flag(PacketFlag::Reliable);
            packet.base.flags.set_flag(PacketFlag::NeedsAck);
            packet.base.flags.set_flag(PacketFlag::HasSize);
            packet.supported_functions = 4;
            packet.maximum_substream_id = 0;
            packet.initial_sequence_id = 0xabcd;
            packet.base.payload = vec![0xaa];
            packet.base.session_id = 1;

            let result: Vec<u8> = packet.to_bytes(flags_version, &context);
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x1f, 0x01, 0x00, 0x00, 0x00, 0xe1, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x15, 0xab, 0x64, 0x8a, 0xc2, 0xea, 0xcd, 0xa7, 0x25, 0x20, 0x19, 0x6f, 0x58, 0x0e,
                0xea, 0x14, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x02,
                0xcd, 0xab, 0x04, 0x01, 0x00, 0xaa,
            ];
            assert_eq!(result, expected_result);
        }
    }

    mod data {
        use super::*;

        #[test]
        fn should_decode_packet() {
            let bytes = vec![
                0xea, 0xd0, 0x01, 0x03, 0x11, 0x00, 0x00, 0x00, 0xe2, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x41, 0xbd, 0xb8, 0xf8, 0x3f, 0x68, 0xdc, 0x5d, 0x04, 0x67, 0x4a, 0xee, 0x5b, 0xec,
                0x04, 0x0d, 0x02, 0x01, 0x00, 0xd3, 0x18, 0x89, 0x41, 0x09, 0x36, 0x5c, 0x3b, 0x8b,
                0x04, 0x1c, 0x65, 0x55, 0x6d, 0x91, 0x6e, 0xc4,
            ];

            let flags_version = 1;
            let context = SignatureContext::default();
            let packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            assert_eq!(packet.base.packet_type, PacketType::Data);
            assert_eq!(packet.base.flags.reliable(), true);
            assert_eq!(packet.base.flags.needs_ack(), true);
            assert_eq!(packet.base.flags.has_size(), true);
            assert_eq!(packet.base.session_id, 1);
            assert_eq!(packet.base.fragment_id, 0);
            assert_eq!(
                packet.base.payload,
                vec![
                    0xd3, 0x18, 0x89, 0x41, 0x09, 0x36, 0x5c, 0x3b, 0x8b, 0x04, 0x1c, 0x65, 0x55,
                    0x6d, 0x91, 0x6e, 0xc4
                ]
            );
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.base.packet_type = PacketType::Data;
            packet.base.flags.clear_flags();
            packet.base.flags.set_flag(PacketFlag::Reliable);
            packet.base.flags.set_flag(PacketFlag::NeedsAck);
            packet.base.flags.set_flag(PacketFlag::HasSize);
            packet.base.session_id = 1;
            packet.base.fragment_id = 0;
            packet.base.payload = vec![
                0x0d, 0x00, 0x00, 0x00, 0xaa, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x03,
                0x03, 0x03, 0x03,
            ];

            let result: Vec<u8> = packet.to_bytes(flags_version, &context);
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x03, 0x11, 0x00, 0x00, 0x00, 0xe2, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x7a, 0xde, 0xd4, 0xa9, 0xac, 0x49, 0x08, 0xcf, 0x5d, 0x93, 0xbb, 0x4f, 0x52, 0xec,
                0x81, 0xa3, 0x02, 0x01, 0x00, 0x0d, 0x00, 0x00, 0x00, 0xaa, 0x01, 0x01, 0x01, 0x01,
                0x02, 0x02, 0x02, 0x02, 0x03, 0x03, 0x03, 0x03,
            ];
            assert_eq!(result, expected_result);
        }
    }

    mod disconnect {
        use super::*;

        #[test]
        fn should_decode_packet() {
            let bytes = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe3, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x35, 0x74, 0x21, 0x30, 0x50, 0xde, 0x6d, 0xd9, 0x1d, 0xdc, 0xa3, 0x8b, 0xf5, 0x7a,
                0x5b, 0x10,
            ];
            let flags_version = 1;
            let context = SignatureContext::default();
            let packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            assert_eq!(packet.base.packet_type, PacketType::Disconnect);
            assert_eq!(packet.base.flags.reliable(), true);
            assert_eq!(packet.base.flags.needs_ack(), true);
            assert_eq!(packet.base.flags.has_size(), true);
            assert_eq!(packet.base.session_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.base.packet_type = PacketType::Disconnect;
            packet.base.flags.clear_flags();
            packet.base.flags.set_flag(PacketFlag::Reliable);
            packet.base.flags.set_flag(PacketFlag::NeedsAck);
            packet.base.flags.set_flag(PacketFlag::HasSize);
            packet.base.session_id = 1;

            let result: Vec<u8> = packet.to_bytes(flags_version, &context);
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe3, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x35, 0x74, 0x21, 0x30, 0x50, 0xde, 0x6d, 0xd9, 0x1d, 0xdc, 0xa3, 0x8b, 0xf5, 0x7a,
                0x5b, 0x10,
            ];
            assert_eq!(result, expected_result);
        }
    }

    mod ping {
        use super::*;

        #[test]
        fn should_decode_packet() {
            let bytes = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc4, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x27, 0x0f, 0x9e, 0xb1, 0x07, 0xda, 0x84, 0x11, 0x88, 0x89, 0x2b, 0x81, 0x92, 0xad,
                0x91, 0x2b,
            ];
            let flags_version = 1;
            let context = SignatureContext::default();
            let packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            assert_eq!(packet.base.packet_type, PacketType::Ping);
            assert_eq!(packet.base.flags.needs_ack(), true);
            assert_eq!(packet.base.flags.has_size(), true);
            assert_eq!(packet.base.session_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.base.packet_type = PacketType::Ping;
            packet.base.flags.clear_flags();
            packet.base.flags.set_flag(PacketFlag::NeedsAck);
            packet.base.flags.set_flag(PacketFlag::HasSize);
            packet.base.session_id = 1;

            let result: Vec<u8> = packet.to_bytes(flags_version, &context);
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc4, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x27, 0x0f, 0x9e, 0xb1, 0x07, 0xda, 0x84, 0x11, 0x88, 0x89, 0x2b, 0x81, 0x92, 0xad,
                0x91, 0x2b,
            ];
            assert_eq!(result, expected_result);
        }
    }
}
