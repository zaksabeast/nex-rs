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
            error_code = error_code | ERROR_MASK;
        }

        self.success = 0;
        self.error_code = error_code;
    }
}

impl EndianRead for RMCResponse {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for RMCResponse {
    fn get_size(&self) -> usize {
        // 16 is when including custom id
        let mut base = if self.protocol_id == 0x7f {
            14
        } else {
            16
        };

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