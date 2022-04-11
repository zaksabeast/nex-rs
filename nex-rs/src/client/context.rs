use super::ClientConnectionResult;
use crate::{
    counter::Counter,
    crypto::{rc4::Rc4, CryptResult},
    packet::{Packet, PacketType, PacketV1, SignatureContext},
};
use getset::{CopyGetters, Getters};

#[derive(Clone, CopyGetters, Getters)]
#[getset(skip)]
pub struct ClientContext {
    #[getset(get_copy = "pub")]
    pub(super) flags_version: u32,
    #[getset(get_copy = "pub")]
    pub(super) signature_base: u32,
    pub(super) cipher: Rc4,
    pub(super) decipher: Rc4,
    pub(super) prudp_version: u32,
    pub(super) sequence_id_in: Counter,
    pub(super) sequence_id_out: Counter,
    pub(super) signature_context: SignatureContext,
}

impl ClientContext {
    pub fn new(flags_version: u32, prudp_version: u32, access_key: &str) -> Self {
        Self {
            flags_version,
            prudp_version,
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            signature_context: SignatureContext::new(access_key),
            ..Default::default()
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> CryptResult<Vec<u8>> {
        self.cipher.encrypt(data)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> CryptResult<Vec<u8>> {
        self.decipher.decrypt(data)
    }

    pub fn get_sequence_id_in(&self) -> u16 {
        self.sequence_id_in
            .value()
            .try_into()
            .expect("Sequence id out does not fit into u16")
    }

    pub fn increment_sequence_id_in(&mut self) -> u16 {
        self.sequence_id_in
            .increment()
            .try_into()
            .expect("Sequence id out does not fit into u16")
    }

    pub fn increment_sequence_id_out(&mut self) -> u16 {
        self.sequence_id_out
            .increment()
            .try_into()
            .expect("Sequence id out does not fit into u16")
    }

    pub(super) fn can_decrypt_packet(&self, packet: &PacketV1) -> ClientConnectionResult<()> {
        if packet.get_packet_type() != PacketType::Data {
            return Err("Only data packets can have payloads".into());
        }

        if packet.get_flags().multi_ack() {
            return Err("Ack packets can not hold payloads".into());
        }

        if packet.get_sequence_id() != self.get_sequence_id_in() {
            return Err("Tried to decode a packet out of order".into());
        }

        Ok(())
    }

    pub(super) fn decrypt_packet(&mut self, packet: &PacketV1) -> ClientConnectionResult<Vec<u8>> {
        self.can_decrypt_packet(packet)?;
        self.decipher
            .decrypt(packet.get_payload())
            .map_err(|error| error.into())
    }

    fn can_encrypt_packet(&self, packet: &PacketV1) -> Result<(), &'static str> {
        if packet.get_packet_type() != PacketType::Data {
            return Err("Only data packets can have payloads");
        }

        if packet.get_flags().multi_ack() {
            return Err("Ack packets can not hold payloads");
        }

        if packet.get_payload().is_empty() {
            return Err("Cannot encode an empty payload");
        }

        Ok(())
    }

    pub(super) fn encrypt_packet(&mut self, packet: &mut PacketV1) {
        if self.can_encrypt_packet(packet).is_ok() {
            let payload = self.cipher.encrypt(packet.get_payload()).unwrap();
            packet.set_payload(payload);
        }
    }
}

impl Default for ClientContext {
    fn default() -> Self {
        Self {
            cipher: Rc4::new(&[0]),
            decipher: Rc4::new(&[0]),
            flags_version: 1,
            prudp_version: 1,
            signature_base: 0,
            sequence_id_in: Counter::default(),
            sequence_id_out: Counter::default(),
            signature_context: SignatureContext::default(),
        }
    }
}
