use super::Packet;

pub struct PacketV1<'a> {
    packet: Packet<'a>,
    magic: Vec<u8>,
    substream_id: u8,
    supported_functions: u32,
    initial_sequence_id: u16,
    maximum_substream_id: u8,
}

impl<'a> PacketV1<'a> {
    pub fn set_substream_id(&self, substream_id: u8) {
        unimplemented!()
    }

    pub fn substream_id(&self) -> u8 {
        unimplemented!()
    }

    pub fn set_supported_functions(&self, supported_functions: u32) {
        unimplemented!()
    }

    pub fn supported_functions(&self) -> u32 {
        unimplemented!()
    }

    pub fn set_initial_sequence_id(&self, initial_sequence_id: u16) {
        unimplemented!()
    }

    pub fn initial_sequence_id(&self) -> u16 {
        unimplemented!()
    }

    pub fn set_maximum_substream_id(&self, maximum_substream_id: u8) {
        unimplemented!()
    }

    pub fn maximum_substream_id(&self) -> u8 {
        unimplemented!()
    }

    pub fn decode() {
        unimplemented!()
    }

    pub fn decode_options(&self, options: Vec<u8>) {
        unimplemented!()
    }

    pub fn encode_options(&self) -> Vec<u8> {
        unimplemented!()
    }

    pub fn calculate_signature(
        &self,
        header: Vec<u8>,
        connection_signature: Vec<u8>,
        options: Vec<u8>,
        payload: Vec<u8>,
    ) -> Vec<u8> {
        unimplemented!()
    }
}
