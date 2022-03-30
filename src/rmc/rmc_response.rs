use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

const ERROR_MASK: u32 = 1 << 31;

#[derive(Default, Debug)]
pub struct RMCResponse {
    pub protocol_id: u8,
    pub custom_id: u16,
    pub success: u8,
    pub call_id: u32,
    pub method_id: u32,
    pub data: Vec<u8>,
    pub error_code: u32,
}

impl RMCResponse {
    pub fn set_success(&mut self, method_id: u32, data: &[u8]) {
        self.success = 1;
        self.method_id = method_id;
        self.data = data.to_vec();
    }

    pub fn set_error(&mut self, mut error_code: u32) {
        if error_code & ERROR_MASK == 0 {
            error_code |= ERROR_MASK;
        }

        self.success = 0;
        self.error_code = error_code;
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

        let success = stream.read_stream_le::<u8>()?;

        let base = if protocol_id == 0x7f { 16 } else { 14 };

        let rmc_response = if success == 1 {
            Self {
                protocol_id,
                custom_id,
                success,
                call_id: stream.read_stream_le()?,
                method_id: stream.read_stream_le()?,
                data: stream.default_read_byte_stream(bytes_len - base),
                error_code: 0,
            }
        } else {
            Self {
                protocol_id,
                custom_id,
                success,
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

        if self.success == 1 {
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

        if self.success == 1 {
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
