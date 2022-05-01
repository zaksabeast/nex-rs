use super::{header::PacketV1Header, options::PacketV1Options};
use crate::packet::{
    Error, Packet, PacketFlag, PacketFlags, PacketResult, PacketType, SignatureContext,
};
use hmac::{Hmac, Mac};
use md5::Md5;
use no_std_io::{Cursor, Reader, StreamContainer, StreamReader, StreamWriter};

#[derive(Debug, Default)]
pub struct PacketV1 {
    flags_version: u32,
    header: PacketV1Header,
    signature: Vec<u8>,
    options: PacketV1Options,
    payload: Vec<u8>,
}

impl Packet for PacketV1 {
    const VERSION: u8 = 1;

    fn to_bytes(self: &PacketV1, context: &SignatureContext) -> Vec<u8> {
        let raw_options = self
            .options
            .as_bytes(&self.header.packet_type(self.flags_version));

        let options_len: u8 = raw_options
            .len()
            .try_into()
            .expect("Options length is too large");
        let payload_size: u16 = self
            .payload
            .len()
            .try_into()
            .expect("Payload length is too large");

        let mut header = self.header;
        header.set_options_length(options_len);
        header.set_payload_size(payload_size);

        let mut stream = StreamContainer::new(vec![]);
        stream.checked_write_stream_le(&header);

        let signature = Self::calculate_signature(
            &stream.get_slice()[2..14].try_into().unwrap(),
            &self.payload,
            context.client_connection_signature(),
            &raw_options,
            context,
        );

        stream.checked_write_stream_bytes(&signature);

        if options_len > 0 {
            stream.checked_write_stream_bytes(&raw_options);
        }

        if !self.payload.is_empty() {
            stream.checked_write_stream_bytes(&self.payload);
        }

        stream.into_raw()
    }

    fn get_source(&self) -> u8 {
        self.header.source()
    }
    fn set_source(&mut self, value: u8) {
        self.header.set_source(value);
    }

    fn get_destination(&self) -> u8 {
        self.header.destination()
    }
    fn set_destination(&mut self, value: u8) {
        self.header.set_destination(value);
    }

    fn get_packet_type(&self) -> PacketType {
        self.header.packet_type(self.flags_version)
    }
    fn set_packet_type(&mut self, value: PacketType) {
        self.header.set_packet_type(self.flags_version, value);
    }

    fn get_flags(&self) -> PacketFlags {
        self.header.flags(self.flags_version)
    }
    fn set_flags(&mut self, value: PacketFlags) {
        self.header.set_flags(self.flags_version, value);
    }

    fn get_session_id(&self) -> u8 {
        self.header.session_id()
    }
    fn set_session_id(&mut self, value: u8) {
        self.header.set_session_id(value);
    }

    fn get_signature(&self) -> &[u8] {
        &self.signature
    }
    fn set_signature(&mut self, value: Vec<u8>) {
        self.signature = value;
    }

    fn get_sequence_id(&self) -> u16 {
        self.header.sequence_id()
    }
    fn set_sequence_id(&mut self, value: u16) {
        self.header.set_sequence_id(value);
    }

    fn get_connection_signature(&self) -> &[u8] {
        &self.options.connection_signature
    }
    fn set_connection_signature(&mut self, value: Vec<u8>) {
        self.options.connection_signature = value;
    }

    fn get_fragment_id(&self) -> u8 {
        self.options.fragment_id
    }
    fn set_fragment_id(&mut self, value: u8) {
        self.options.fragment_id = value;
    }

    fn get_payload(&self) -> &[u8] {
        &self.payload
    }
    fn set_payload(&mut self, value: Vec<u8>) {
        self.payload = value;
    }
}

impl PacketV1 {
    pub fn new_ping_packet(flags_version: u32) -> Self {
        let mut header = PacketV1Header::default();
        header.set_source(Self::SERVER_ID);
        header.set_destination(Self::CLIENT_ID);
        header.set_packet_type(flags_version, PacketType::Ping);
        header.set_flags(flags_version, PacketFlag::Ack | PacketFlag::Reliable);

        Self {
            header,
            flags_version,
            ..Default::default()
        }
    }

    pub fn new_data_packet(
        session_id: u8,
        connection_signature: Vec<u8>,
        payload: Vec<u8>,
        flags_version: u32,
    ) -> Self {
        let mut header = PacketV1Header::default();
        header.set_session_id(session_id);
        header.set_source(Self::SERVER_ID);
        header.set_destination(Self::CLIENT_ID);
        header.set_packet_type(flags_version, PacketType::Data);
        header.set_flags(
            flags_version,
            PacketFlag::Reliable | PacketFlag::NeedsAck | PacketFlag::HasSize,
        );

        Self {
            header,
            payload,
            options: PacketV1Options {
                connection_signature,
                ..Default::default()
            },
            flags_version,
            ..Default::default()
        }
    }

    pub fn new_disconnect_packet(flags_version: u32) -> Self {
        let mut header = PacketV1Header::default();
        header.set_source(Self::SERVER_ID);
        header.set_destination(Self::CLIENT_ID);
        header.set_packet_type(flags_version, PacketType::Disconnect);
        header.set_flags(
            flags_version,
            PacketFlag::Reliable | PacketFlag::NeedsAck | PacketFlag::HasSize,
        );

        Self {
            header,
            flags_version,
            ..Default::default()
        }
    }

    pub fn new_ack_packet(&self) -> Self {
        let mut header = PacketV1Header::default();
        header.set_source(self.get_destination());
        header.set_destination(self.get_source());
        header.set_packet_type(self.flags_version, self.get_packet_type());
        header.set_flags(self.flags_version, PacketFlag::Ack | PacketFlag::HasSize);
        header.set_substream_id(0);

        Self {
            header,
            options: PacketV1Options {
                fragment_id: self.get_fragment_id(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn read_packet(
        data: Vec<u8>,
        flags_version: u32,
        context: &SignatureContext,
    ) -> PacketResult<Self> {
        let data_len = data.len();

        let mut packet = PacketV1 {
            flags_version,
            ..Self::default()
        };

        if data_len > 0 {
            packet.decode(data, context)?;
        }

        Ok(packet)
    }

    pub fn get_substream_id(&self) -> u8 {
        self.header.substream_id()
    }
    pub fn set_substream_id(&mut self, value: u8) {
        self.header.set_substream_id(value);
    }

    pub fn get_supported_functions(&self) -> u32 {
        self.options.supported_functions
    }
    pub fn set_supported_functions(&mut self, value: u32) {
        self.options.supported_functions = value;
    }

    pub fn get_initial_sequence_id(&self) -> u16 {
        self.options.initial_sequence_id
    }
    pub fn set_initial_sequence_id(&mut self, value: u16) {
        self.options.initial_sequence_id = value;
    }

    pub fn get_maximum_substream_id(&self) -> u8 {
        self.options.maximum_substream_id
    }
    pub fn set_maximum_substream_id(&mut self, value: u8) {
        self.options.maximum_substream_id = value;
    }

    fn decode(&mut self, data: Vec<u8>, context: &SignatureContext) -> PacketResult<()> {
        let data_len = data.len();

        // magic + header + signature
        if data_len < 30 {
            return Err(Error::InvalidSize {
                wanted_size: 30,
                received_size: data_len,
                context: "Data length is less than the smallest possible packet size",
            });
        }

        let mut stream = StreamContainer::new(data.as_slice());

        self.header = stream.default_read_stream_le::<PacketV1Header>();
        self.signature = stream.default_read_byte_stream(16).try_into().unwrap();

        let options_length = usize::from(self.header.options_length());
        let options_end = stream.get_index() + options_length;
        if data_len < options_end {
            return Err(Error::InvalidSize {
                wanted_size: options_end,
                received_size: data_len,
                context: "The options length does not fit into the packet data length",
            });
        }

        let raw_options = stream.default_read_byte_stream(options_length);
        self.options = raw_options.default_read_le(0);

        let payload_size = usize::from(self.header.payload_size());
        if payload_size > 0 {
            self.payload = stream.default_read_byte_stream(payload_size);
        }

        let calculated_signature = Self::calculate_signature(
            &data[2..14].try_into().unwrap(),
            &self.payload,
            context.server_connection_signature(),
            &raw_options,
            context,
        );

        if calculated_signature != self.signature {
            return Err(Error::InvalidSignature {
                calculated_signature,
                found_signature: self.signature.to_vec(),
                packet_type: self.get_packet_type(),
                sequence_id: self.get_sequence_id(),
            });
        }

        Ok(())
    }

    pub fn calculate_signature(
        header: &[u8; 12],
        payload: &[u8],
        connection_signature: &[u8],
        options: &[u8],
        context: &SignatureContext,
    ) -> Vec<u8> {
        let key: &[u8; 16] = context.signature_key();
        let signature_base = context.signature_base();

        // The key being [u8; 16] guarantees we won't run into an error
        let mut mac = Hmac::<Md5>::new_from_slice(key).unwrap();
        mac.update(&header[4..]);
        mac.update(context.session_key());
        mac.update(&signature_base.to_le_bytes());
        mac.update(connection_signature);
        mac.update(options);
        mac.update(payload);
        mac.finalize().into_bytes().to_vec()
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
        let packet = PacketV1::read_packet(bytes.clone(), flags_version, &context)
            .expect("Should have succeeded!");
        let result = packet.to_bytes(&context);
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

            assert_eq!(packet.get_packet_type(), PacketType::Syn);
            assert_eq!(packet.get_flags().needs_ack(), true);
            assert_eq!(packet.get_flags().has_size(), true);
            assert_eq!(packet.options.supported_functions, 4);
            assert_eq!(packet.options.maximum_substream_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.set_packet_type(PacketType::Syn);
            packet.set_flags(PacketFlag::NeedsAck | PacketFlag::HasSize);
            packet.options.supported_functions = 4;
            packet.options.maximum_substream_id = 1;

            let result: Vec<u8> = packet.to_bytes(&context);
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

            assert_eq!(packet.get_packet_type(), PacketType::Connect);
            assert_eq!(packet.get_flags().reliable(), true);
            assert_eq!(packet.get_flags().needs_ack(), true);
            assert_eq!(packet.get_flags().has_size(), true);
            assert_eq!(packet.get_session_id(), 1);
            assert_eq!(packet.options.supported_functions, 4);
            assert_eq!(packet.options.maximum_substream_id, 0);
            assert_eq!(packet.options.initial_sequence_id, 0xabcd);
            assert_eq!(packet.payload, vec![0xaa]);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.set_packet_type(PacketType::Connect);
            packet.set_flags(PacketFlag::Reliable | PacketFlag::NeedsAck | PacketFlag::HasSize);
            packet.set_session_id(1);
            packet.options.supported_functions = 4;
            packet.options.maximum_substream_id = 0;
            packet.options.initial_sequence_id = 0xabcd;
            packet.payload = vec![0xaa];

            let result: Vec<u8> = packet.to_bytes(&context);
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

            assert_eq!(packet.get_packet_type(), PacketType::Data);
            assert_eq!(packet.get_flags().reliable(), true);
            assert_eq!(packet.get_flags().needs_ack(), true);
            assert_eq!(packet.get_flags().has_size(), true);
            assert_eq!(packet.get_session_id(), 1);
            assert_eq!(packet.options.fragment_id, 0);
            assert_eq!(
                packet.payload,
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

            packet.set_packet_type(PacketType::Data);
            packet.set_flags(PacketFlag::Reliable | PacketFlag::NeedsAck | PacketFlag::HasSize);
            packet.set_session_id(1);
            packet.options.fragment_id = 0;
            packet.payload = vec![
                0x0d, 0x00, 0x00, 0x00, 0xaa, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x03,
                0x03, 0x03, 0x03,
            ];

            let result: Vec<u8> = packet.to_bytes(&context);
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

            assert_eq!(packet.get_packet_type(), PacketType::Disconnect);
            assert_eq!(packet.get_flags().reliable(), true);
            assert_eq!(packet.get_flags().needs_ack(), true);
            assert_eq!(packet.get_flags().has_size(), true);
            assert_eq!(packet.get_session_id(), 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.set_packet_type(PacketType::Disconnect);
            packet.set_flags(PacketFlag::Reliable | PacketFlag::NeedsAck | PacketFlag::HasSize);
            packet.set_session_id(1);

            let result: Vec<u8> = packet.to_bytes(&context);
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

            assert_eq!(packet.get_packet_type(), PacketType::Ping);
            assert_eq!(packet.get_flags().needs_ack(), true);
            assert_eq!(packet.get_flags().has_size(), true);
            assert_eq!(packet.get_session_id(), 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();
            let flags_version = 1;
            let context = SignatureContext::default();
            let mut packet = PacketV1::read_packet(bytes, flags_version, &context)
                .expect("Should have succeeded!");

            packet.set_packet_type(PacketType::Ping);
            packet.set_flags(PacketFlag::NeedsAck | PacketFlag::HasSize);
            packet.set_session_id(1);

            let result: Vec<u8> = packet.to_bytes(&context);
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc4, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x27, 0x0f, 0x9e, 0xb1, 0x07, 0xda, 0x84, 0x11, 0x88, 0x89, 0x2b, 0x81, 0x92, 0xad,
                0x91, 0x2b,
            ];
            assert_eq!(result, expected_result);
        }
    }
}
