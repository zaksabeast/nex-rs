#[derive(Debug, Default)]
pub struct PacketV1Options {
    pub(super) supported_functions: u32,
    pub(super) fragment_id: u8,
    pub(super) initial_sequence_id: u16,
    pub(super) maximum_substream_id: u8,
    pub(super) connection_signature: Vec<u8>,
}
