use super::{Packet, PacketFlag, PacketFlags, PacketOption, PacketType};
use crate::{
    client::Client,
    stream::{StreamIn, StreamOut},
};
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
    pub fn new(client: &'a mut Client<'a>, data: Vec<u8>) -> Result<Self, &'static str> {
        let data_len = data.len();

        let mut packet = Self {
            packet: Packet::new(client, data),
            magic: 0,
            substream_id: 0,
            supported_functions: 0,
            initial_sequence_id: 0,
            maximum_substream_id: 0,
        };

        if data_len > 0 {
            packet.decode()?;
        }

        Ok(packet)
    }

    fn decode(&mut self) -> Result<(), &'static str> {
        let data_len = self.packet.data.len();
        let data = self.packet.data.as_slice();

        // magic + header + signature
        if data_len < 30 {
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

        if data_len < stream.get_index() + options_length {
            return Err("Packet specific data size does not match");
        }

        let options = stream.default_read_byte_stream(options_length);

        if payload_size > 0 {
            self.packet.payload = stream.default_read_byte_stream(payload_size);

            if self.packet.packet_type == PacketType::Data && !self.packet.flags.multi_ack() {
                let decipher = self.packet.sender.get_decipher();
                decipher.encrypt(&mut self.packet.payload);
                self.packet.rmc_request = self.packet.payload.as_slice().try_into()?;
            }
        }

        let calculated_signature = self.calculate_signature(&options)?;

        if calculated_signature == self.packet.signature {
            return Err("Calculated signature did not match");
        }

        self.decode_options(&options)?;

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

    pub fn calculate_signature(&self, options: &[u8]) -> Result<Vec<u8>, &'static str> {
        if self.packet.data.len() < 14 {
            return Err("Packet data length is too small");
        }

        let header = &self.packet.data[6..14];
        let connection_signature = &self.packet.connection_signature;
        let payload = &self.packet.payload;
        let key = self.packet.sender.get_signature_key();
        let signature_base = self.packet.sender.get_signature_base();

        let mut mac = Hmac::<Md5>::new_from_slice(key).map_err(|_| "Invalid hamc key size")?;
        mac.update(&header[4..]);
        mac.update(self.packet.sender.get_session_key());
        mac.update(&signature_base.to_le_bytes());
        mac.update(connection_signature);
        mac.update(options);
        mac.update(payload);
        Ok(mac.finalize().into_bytes().to_vec())
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

        let signature = packet
            .calculate_signature(&options)
            .expect("Signature could not be calculated");
        stream.checked_write_stream_bytes(&signature);

        if options_len > 0 {
            stream.checked_write_stream_bytes(&options);
        }

        if !packet.packet.payload.is_empty() {
            stream.checked_write_stream_bytes(&packet.packet.payload);
        }

        stream.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::Server;

    const BASE_PACKET: [u8; 57] = [
        0xea, 0xd0, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x8a,
        0xb1, 0x6d, 0x53, 0x40, 0x5d, 0xf1, 0xa0, 0xc8, 0x9a, 0xdd, 0x37, 0xe3, 0xcf, 0xf5, 0xaa,
        0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, 0x00,
    ];

    #[test]
    fn should_encode_and_decode() {
        let bytes = BASE_PACKET.to_vec();

        let mut server = Server::new();
        let mut client = Client::new(&mut server);

        let packet = PacketV1::new(&mut client, bytes.clone()).expect("Should have succeeded!");

        let result: Vec<u8> = packet.into();
        assert_eq!(result, bytes);
    }

    mod syn {
        use super::*;

        #[test]
        fn should_decode_packet() {
            let bytes = vec![
                0xea, 0xd0, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x8e, 0x8a, 0xa3, 0x5e, 0xda, 0xe9, 0xe6, 0xfc, 0xc9, 0xa0, 0xcc, 0xdc, 0x7e, 0x9c,
                0x88, 0x81, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01,
                0x01,
            ];

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let packet = PacketV1::new(&mut client, bytes.clone()).expect("Should have succeeded!");

            assert_eq!(packet.packet.packet_type, PacketType::Syn);
            assert_eq!(packet.packet.flags.needs_ack(), true);
            assert_eq!(packet.packet.flags.has_size(), true);
            assert_eq!(packet.supported_functions, 4);
            assert_eq!(packet.maximum_substream_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let mut packet = PacketV1::new(&mut client, bytes).expect("Should have succeeded!");
            packet.packet.packet_type = PacketType::Syn;
            packet.packet.flags.clear_flags();
            packet.packet.flags.set_flag(PacketFlag::NeedsAck);
            packet.packet.flags.set_flag(PacketFlag::HasSize);
            packet.supported_functions = 4;
            packet.maximum_substream_id = 1;

            let result: Vec<u8> = packet.into();
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x8e, 0x8a, 0xa3, 0x5e, 0xda, 0xe9, 0xe6, 0xfc, 0xc9, 0xa0, 0xcc, 0xdc, 0x7e, 0x9c,
                0x88, 0x81, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
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
                0x28, 0x66, 0xa0, 0x43, 0x3c, 0xcd, 0x20, 0xcb, 0xac, 0x2f, 0x29, 0x68, 0x5f, 0x90,
                0x97, 0x75, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x02,
                0xcd, 0xab, 0x04, 0x01, 0x00, 0xaa,
            ];

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let packet = PacketV1::new(&mut client, bytes.clone()).expect("Should have succeeded!");

            assert_eq!(packet.packet.packet_type, PacketType::Connect);
            assert_eq!(packet.packet.flags.reliable(), true);
            assert_eq!(packet.packet.flags.needs_ack(), true);
            assert_eq!(packet.packet.flags.has_size(), true);
            assert_eq!(packet.supported_functions, 4);
            assert_eq!(packet.maximum_substream_id, 0);
            assert_eq!(packet.initial_sequence_id, 0xabcd);
            assert_eq!(packet.packet.payload, vec![0xaa]);
            assert_eq!(packet.packet.session_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let mut packet = PacketV1::new(&mut client, bytes).expect("Should have succeeded!");
            packet.packet.packet_type = PacketType::Connect;
            packet.packet.flags.clear_flags();
            packet.packet.flags.set_flag(PacketFlag::Reliable);
            packet.packet.flags.set_flag(PacketFlag::NeedsAck);
            packet.packet.flags.set_flag(PacketFlag::HasSize);
            packet.supported_functions = 4;
            packet.maximum_substream_id = 0;
            packet.initial_sequence_id = 0xabcd;
            packet.packet.payload = vec![0xaa];
            packet.packet.session_id = 1;

            let result: Vec<u8> = packet.into();
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x1f, 0x01, 0x00, 0x00, 0x00, 0xe1, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x28, 0x66, 0xa0, 0x43, 0x3c, 0xcd, 0x20, 0xcb, 0xac, 0x2f, 0x29, 0x68, 0x5f, 0x90,
                0x97, 0x75, 0x00, 0x04, 0x04, 0x00, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x00,
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
                0x1f, 0x9a, 0x3b, 0xb2, 0x89, 0x33, 0x50, 0x16, 0x4e, 0x79, 0xdd, 0x12, 0xd1, 0xcd,
                0xd4, 0xda, 0x02, 0x01, 0x00, 0xd3, 0x18, 0x89, 0x41, 0x09, 0x36, 0x5c, 0x3b, 0x8b,
                0x04, 0x1c, 0x65, 0x55, 0x6d, 0x91, 0x6e, 0xc4,
            ];

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let packet = PacketV1::new(&mut client, bytes.clone()).expect("Should have succeeded!");

            assert_eq!(packet.packet.packet_type, PacketType::Data);
            assert_eq!(packet.packet.flags.reliable(), true);
            assert_eq!(packet.packet.flags.needs_ack(), true);
            assert_eq!(packet.packet.flags.has_size(), true);
            assert_eq!(packet.packet.session_id, 1);
            assert_eq!(packet.packet.fragment_id, 0);
            assert_eq!(
                packet.packet.payload,
                vec![
                    0x0d, 0x00, 0x00, 0x00, 0xaa, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02,
                    0x03, 0x03, 0x03, 0x03,
                ]
            );
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let mut packet = PacketV1::new(&mut client, bytes).expect("Should have succeeded!");
            packet.packet.packet_type = PacketType::Data;
            packet.packet.flags.clear_flags();
            packet.packet.flags.set_flag(PacketFlag::Reliable);
            packet.packet.flags.set_flag(PacketFlag::NeedsAck);
            packet.packet.flags.set_flag(PacketFlag::HasSize);
            packet.packet.session_id = 1;
            packet.packet.fragment_id = 0;
            packet.packet.payload = vec![
                0x0d, 0x00, 0x00, 0x00, 0xaa, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x03,
                0x03, 0x03, 0x03,
            ];

            let result: Vec<u8> = packet.into();
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x03, 0x11, 0x00, 0x00, 0x00, 0xe2, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x1f, 0x9a, 0x3b, 0xb2, 0x89, 0x33, 0x50, 0x16, 0x4e, 0x79, 0xdd, 0x12, 0xd1, 0xcd,
                0xd4, 0xda, 0x02, 0x01, 0x00, 0xd3, 0x18, 0x89, 0x41, 0x09, 0x36, 0x5c, 0x3b, 0x8b,
                0x04, 0x1c, 0x65, 0x55, 0x6d, 0x91, 0x6e, 0xc4,
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
                0x10, 0xa3, 0x3d, 0xac, 0x5f, 0x58, 0x97, 0x3f, 0x8e, 0x83, 0xb7, 0x23, 0x16, 0xde,
                0xc8, 0x47,
            ];

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let packet = PacketV1::new(&mut client, bytes.clone()).expect("Should have succeeded!");

            assert_eq!(packet.packet.packet_type, PacketType::Disconnect);
            assert_eq!(packet.packet.flags.reliable(), true);
            assert_eq!(packet.packet.flags.needs_ack(), true);
            assert_eq!(packet.packet.flags.has_size(), true);
            assert_eq!(packet.packet.session_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let mut packet = PacketV1::new(&mut client, bytes).expect("Should have succeeded!");
            packet.packet.packet_type = PacketType::Disconnect;
            packet.packet.flags.clear_flags();
            packet.packet.flags.set_flag(PacketFlag::Reliable);
            packet.packet.flags.set_flag(PacketFlag::NeedsAck);
            packet.packet.flags.set_flag(PacketFlag::HasSize);
            packet.packet.session_id = 1;

            let result: Vec<u8> = packet.into();
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe3, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x10, 0xa3, 0x3d, 0xac, 0x5f, 0x58, 0x97, 0x3f, 0x8e, 0x83, 0xb7, 0x23, 0x16, 0xde,
                0xc8, 0x47,
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
                0x10, 0xa3, 0x3d, 0xac, 0x5f, 0x58, 0x97, 0x3f, 0x8e, 0x83, 0xb7, 0x23, 0x16, 0xde,
                0xc8, 0x47,
            ];

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let packet = PacketV1::new(&mut client, bytes.clone()).expect("Should have succeeded!");

            assert_eq!(packet.packet.packet_type, PacketType::Ping);
            assert_eq!(packet.packet.flags.needs_ack(), true);
            assert_eq!(packet.packet.flags.has_size(), true);
            assert_eq!(packet.packet.session_id, 1);
        }

        #[test]
        fn should_encode_packet() {
            let bytes = BASE_PACKET.to_vec();

            let mut server = Server::new();
            let mut client = Client::new(&mut server);

            let mut packet = PacketV1::new(&mut client, bytes).expect("Should have succeeded!");
            packet.packet.packet_type = PacketType::Ping;
            packet.packet.flags.clear_flags();
            packet.packet.flags.set_flag(PacketFlag::NeedsAck);
            packet.packet.flags.set_flag(PacketFlag::HasSize);
            packet.packet.session_id = 1;

            let result: Vec<u8> = packet.into();
            let expected_result = vec![
                0xea, 0xd0, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc4, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x10, 0xa3, 0x3d, 0xac, 0x5f, 0x58, 0x97, 0x3f, 0x8e, 0x83, 0xb7, 0x23, 0x16, 0xde,
                0xc8, 0x47,
            ];
            assert_eq!(result, expected_result);
        }
    }
}
