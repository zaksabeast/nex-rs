use crate::packet::{PacketOption, PacketType};
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default, Clone)]
pub struct PacketV1Options {
    pub(super) supported_functions: u32,
    pub(super) fragment_id: u8,
    pub(super) initial_sequence_id: u16,
    pub(super) maximum_substream_id: u8,
    pub(super) connection_signature: Vec<u8>,
}

impl PacketV1Options {
    fn syn_options(&self) -> SynOptions {
        SynOptions {
            supported_functions: self.supported_functions,
            connection_signature: self.connection_signature.clone(),
            maximum_substream_id: self.maximum_substream_id,
        }
    }

    fn connect_options(&self) -> ConnectOptions {
        ConnectOptions {
            supported_functions: self.supported_functions,
            connection_signature: self.connection_signature.clone(),
            initial_sequence_id: self.initial_sequence_id,
            maximum_substream_id: self.maximum_substream_id,
        }
    }

    fn data_options(&self) -> DataOptions {
        DataOptions {
            fragment_id: self.fragment_id,
        }
    }

    pub fn as_bytes(&self, packet_type: &PacketType) -> Vec<u8> {
        let mut stream = StreamContainer::new(vec![]);

        match packet_type {
            PacketType::Syn => {
                stream.checked_write_stream_le(&self.syn_options());
            }
            PacketType::Connect => {
                stream.checked_write_stream_le(&self.connect_options());
            }
            PacketType::Data => {
                stream.checked_write_stream_le(&self.data_options());
            }
            _ => {}
        };

        stream.into_raw()
    }
}

#[derive(Debug, Default)]
pub struct SynOptions {
    supported_functions: u32,
    connection_signature: Vec<u8>,
    maximum_substream_id: u8,
}

impl EndianWrite for SynOptions {
    fn get_size(&self) -> usize {
        PacketOption::SupportedFunctions.write_size()
            + PacketOption::ConnectionSignature.write_size()
            + PacketOption::MaxSubstreamId.write_size()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);

        stream.write_stream_le(&(PacketOption::SupportedFunctions as u8))?;
        stream.write_stream_le(&PacketOption::SupportedFunctions.value_size())?;
        stream.write_stream_le(&self.supported_functions)?;

        stream.write_stream_le(&(PacketOption::ConnectionSignature as u8))?;
        stream.write_stream_le(&PacketOption::ConnectionSignature.value_size())?;
        stream.write_stream_bytes(&self.connection_signature)?;

        stream.write_stream_le(&(PacketOption::MaxSubstreamId as u8))?;
        stream.write_stream_le(&PacketOption::MaxSubstreamId.value_size())?;
        stream.write_stream_le(&self.maximum_substream_id)?;

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct ConnectOptions {
    supported_functions: u32,
    connection_signature: Vec<u8>,
    initial_sequence_id: u16,
    maximum_substream_id: u8,
}

impl EndianWrite for ConnectOptions {
    fn get_size(&self) -> usize {
        PacketOption::SupportedFunctions.write_size()
            + PacketOption::ConnectionSignature.write_size()
            + PacketOption::InitialSequenceId.write_size()
            + PacketOption::MaxSubstreamId.write_size()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);

        stream.write_stream_le(&(PacketOption::SupportedFunctions as u8))?;
        stream.write_stream_le(&PacketOption::SupportedFunctions.value_size())?;
        stream.write_stream_le(&self.supported_functions)?;

        stream.write_stream_le(&(PacketOption::ConnectionSignature as u8))?;
        stream.write_stream_le(&PacketOption::ConnectionSignature.value_size())?;
        stream.write_stream_bytes(&self.connection_signature)?;

        stream.write_stream_le(&(PacketOption::InitialSequenceId as u8))?;
        stream.write_stream_le(&PacketOption::InitialSequenceId.value_size())?;
        stream.write_stream_le(&self.initial_sequence_id)?;

        stream.write_stream_le(&(PacketOption::MaxSubstreamId as u8))?;
        stream.write_stream_le(&PacketOption::MaxSubstreamId.value_size())?;
        stream.write_stream_le(&self.maximum_substream_id)?;

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}

#[derive(Debug, Default)]
pub struct DataOptions {
    fragment_id: u8,
}

impl EndianWrite for DataOptions {
    fn get_size(&self) -> usize {
        PacketOption::FragmentId.write_size()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);

        stream.write_stream_le(&(PacketOption::FragmentId as u8))?;
        stream.write_stream_le(&PacketOption::FragmentId.value_size())?;
        stream.write_stream_le(&self.fragment_id)?;

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
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
                    result.supported_functions = stream.read_stream_le()?;
                }
                PacketOption::ConnectionSignature => {
                    result.connection_signature = stream.read_byte_stream(option_size)?;
                }
                PacketOption::FragmentId => {
                    result.fragment_id = stream.read_stream_le()?;
                }
                PacketOption::InitialSequenceId => {
                    result.initial_sequence_id = stream.read_stream_le()?;
                }
                PacketOption::MaxSubstreamId => {
                    result.maximum_substream_id = stream.read_stream_le()?;
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
