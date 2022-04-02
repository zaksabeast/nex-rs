use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

const ERROR_MASK: u32 = 1 << 31;

#[derive(Default, Debug)]
pub struct RMCResponse {
    protocol_id: u8,
    custom_id: u16,
    is_success: bool,
    call_id: u32,
    method_id: u32,
    data: Vec<u8>,
    error_code: u32,
}

impl RMCResponse {
    pub fn new_success(
        protocol_id: u8,
        method_id: impl Into<u32>,
        call_id: u32,
        data: Vec<u8>,
    ) -> Self {
        Self {
            data,
            protocol_id,
            method_id: method_id.into(),
            call_id,
            is_success: true,
            error_code: 0,
            custom_id: 0,
        }
    }

    pub fn new_error(
        protocol_id: u8,
        method_id: impl Into<u32>,
        call_id: u32,
        mut error_code: u32,
    ) -> Self {
        if error_code & ERROR_MASK == 0 {
            error_code |= ERROR_MASK;
        }

        Self {
            protocol_id,
            method_id: method_id.into(),
            call_id,
            error_code,
            data: vec![],
            is_success: false,
            custom_id: 0,
        }
    }
}

impl EndianRead for RMCResponse {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let bytes_len = bytes.len();
        if bytes_len < 13 {
            return Err(Error::InvalidRead {
                message: "Invalid RMCResponse size",
            });
        }

        let mut stream = StreamContainer::new(bytes);

        let size: usize =
            stream
                .read_stream_le::<u32>()?
                .try_into()
                .map_err(|_| Error::InvalidRead {
                    message: "RMCResponse size does not fit into usize",
                })?;

        if size != bytes_len - 4 {
            return Err(Error::InvalidRead {
                message: "RMCResponse data size does not match",
            });
        }

        let protocol_id = stream.read_stream_le::<u8>()?;
        let custom_id = if protocol_id == 0x7f {
            stream.read_stream_le()?
        } else {
            0
        };

        let is_success = stream.read_stream_le()?;

        let base = if protocol_id == 0x7f { 16 } else { 14 };

        let rmc_response = if is_success {
            Self {
                protocol_id,
                custom_id,
                is_success,
                call_id: stream.read_stream_le()?,
                method_id: stream.read_stream_le()?,
                data: stream.default_read_byte_stream(bytes_len - base),
                error_code: 0,
            }
        } else {
            Self {
                protocol_id,
                custom_id,
                is_success,
                error_code: stream.read_stream_le()?,
                call_id: stream.read_stream_le()?,
                method_id: 0,
                data: vec![],
            }
        };
        Ok(ReadOutput::new(rmc_response, stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for RMCResponse {
    fn get_size(&self) -> usize {
        // 16 is when including custom id
        let mut base = if self.protocol_id == 0x7f { 16 } else { 14 };

        if self.is_success {
            base += 8 + self.data.len();
        }

        base
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        let data_size: u32 = (self.get_size() - 4)
            .try_into()
            .map_err(|_| Error::InvalidWrite {
                message: "RMCRequest size does not fit into u32",
            })?;

        stream.write_stream_le(&data_size)?;
        stream.write_stream_le(&self.protocol_id)?;

        if self.protocol_id == 0x7f {
            stream.write_stream_le(&self.custom_id)?;
        }

        stream.write_stream_le(&self.is_success)?;

        if self.is_success {
            stream.write_stream_le(&self.call_id)?;
            stream.write_stream_le(&(self.method_id | 0x8000))?;
            stream.write_stream_bytes(&self.data)?;
        } else {
            stream.write_stream_le(&self.error_code)?;
            stream.write_stream_le(&self.call_id)?;
        }

        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
