use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

pub struct NexBuffer(Vec<u8>);

impl EndianRead for NexBuffer {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let length: usize =
            stream
                .read_stream_le::<u32>()?
                .try_into()
                .map_err(|_| Error::InvalidRead {
                    message: "NexBuffer length does not fit into usize",
                })?;

        let bytes = stream.read_byte_stream(length)?;
        Ok(ReadOutput::new(NexBuffer(bytes), stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for NexBuffer {
    fn get_size(&self) -> usize {
        self.0.len() + 1
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let len: u32 = self.0.len().try_into().map_err(|_| Error::InvalidWrite {
            message: "NexBuffer length does not fit into u32",
        })?;

        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&len)?;
        stream.write_stream_bytes(&self.0)?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl From<NexBuffer> for Vec<u8> {
    fn from(nex: NexBuffer) -> Self {
        nex.0
    }
}

impl From<Vec<u8>> for NexBuffer {
    fn from(raw: Vec<u8>) -> Self {
        NexBuffer(raw)
    }
}
