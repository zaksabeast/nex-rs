use crate::packet::PacketOption;
use no_std_io::{Cursor, EndianRead, Error, ReadOutput, StreamContainer, StreamReader};

#[derive(Debug, Default)]
pub struct PacketV1Options {
    pub(super) supported_functions: u32,
    pub(super) fragment_id: u8,
    pub(super) initial_sequence_id: u16,
    pub(super) maximum_substream_id: u8,
    pub(super) connection_signature: Vec<u8>,
}

impl EndianRead for PacketV1Options {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let options_len = bytes.len();

        let mut result = PacketV1Options::default();

        let mut i = 0;
        while i < options_len {
            let option_type: PacketOption =
                stream
                    .read_stream_le::<u8>()?
                    .try_into()
                    .map_err(|_| Error::InvalidRead {
                        message: "Invalid packet option",
                    })?;
            let option_size = usize::from(stream.default_read_stream_le::<u8>());

            match option_type {
                PacketOption::SupportedFunctions => {
                    let lsb = stream.read_byte_stream(option_size)?[0];
                    // TODO: Set nex version
                    // Is this something we want clients controlling?
                    // Should we know this already?
                    result.supported_functions = lsb.into();
                }
                PacketOption::ConnectionSignature => {
                    result.connection_signature = stream.read_byte_stream(option_size)?;
                }
                PacketOption::FragmentId => {
                    result.fragment_id = stream.default_read_stream_le();
                }
                PacketOption::InitialSequenceId => {
                    result.initial_sequence_id = stream.default_read_stream_le();
                }
                PacketOption::MaxSubstreamId => {
                    result.maximum_substream_id = stream.default_read_stream_le();
                }
            }

            i = stream.get_index();
        }

        Ok(ReadOutput::new(result, stream.get_index()))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}
