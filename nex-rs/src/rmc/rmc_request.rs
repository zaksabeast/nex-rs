use crate::route::NexProtocol;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Default, Debug)]
pub struct RMCRequest {
    pub protocol_id: u8,
    pub call_id: u32,
    pub method_id: u32,
    pub parameters: Vec<u8>,
    pub custom_id: u16,
}

impl RMCRequest {
    pub fn is_method<T: NexProtocol + Into<u32>>(&self, method: T) -> bool {
        self.protocol_id == T::PROTOCOL_ID && self.method_id == method.into()
    }
}

impl EndianRead for RMCRequest {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let bytes_len = bytes.len();
        if bytes_len < 13 {
            return Err(Error::InvalidRead {
                message: "Invalid RMCRequest size",
            });
        }

        let mut stream = StreamContainer::new(bytes);

        let size: usize =
            stream
                .read_stream_le::<u32>()?
                .try_into()
                .map_err(|_| Error::InvalidRead {
                    message: "RMCRequest size does not fit into usize",
                })?;

        if size != bytes_len - 4 {
            return Err(Error::InvalidRead {
                message: "RMCRequest data size does not match",
            });
        }

        let protocol_id = stream.read_stream_le::<u8>()? ^ 0x80;
        let custom_id = if protocol_id == 0x7f {
            stream.read_stream_le()?
        } else {
            0
        };

        let base = if protocol_id == 0x7f { 15 } else { 13 };

        let rmc_request = Self {
            protocol_id,
            custom_id,
            call_id: stream.default_read_stream_le(),
            method_id: stream.default_read_stream_le(),
            parameters: stream.default_read_byte_stream(bytes_len - base),
        };

        Ok(ReadOutput::new(rmc_request, stream.get_index()))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for RMCRequest {
    fn get_size(&self) -> usize {
        // 17 is when including custom id
        let base = if self.protocol_id == 0x7f { 17 } else { 13 };

        self.parameters.len() + base
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        let data_size: u32 = (self.get_size() - 4)
            .try_into()
            .map_err(|_| Error::InvalidWrite {
                message: "RMCRequest size does not fit into u32",
            })?;

        stream.write_stream_le(&data_size)?;
        stream.write_stream_le(&(self.protocol_id | 0x80))?;

        if self.protocol_id == 0x7f {
            stream.write_stream_le(&self.custom_id)?;
        }

        stream.write_stream_le(&self.call_id)?;
        stream.write_stream_le(&self.method_id)?;
        stream.write_stream_bytes(&self.parameters)?;

        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
