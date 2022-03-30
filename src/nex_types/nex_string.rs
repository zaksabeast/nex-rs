use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default, Clone)]
pub struct NexString(String);

impl From<NexString> for String {
    fn from(nex: NexString) -> Self {
        nex.0
    }
}

impl From<String> for NexString {
    fn from(raw: String) -> Self {
        NexString(raw)
    }
}

impl EndianWrite for NexString {
    fn get_size(&self) -> usize {
        self.0.len() + 3
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let raw = &self.0;
        let len: u16 = (self.0.len() + 1)
            .try_into()
            .map_err(|_| Error::InvalidWrite {
                message: "String length does not fit into u16",
            })?;

        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&len)?;
        stream.write_stream_bytes(raw.as_bytes())?;
        stream.write_stream(&0u8)?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl EndianRead for NexString {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let length: u16 = stream.read_stream_le()?;
        let read_bytes = stream.read_byte_stream(length.into())?;
        let raw = String::from_utf8(read_bytes).map_err(|_| Error::InvalidRead {
            message: "Bytes weren't valid utf8",
        })?;

        Ok(ReadOutput::new(NexString(raw), stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}
