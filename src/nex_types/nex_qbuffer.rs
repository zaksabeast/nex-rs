use core::mem;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default)]
pub struct NexQBuffer(Vec<u8>);

impl EndianRead for NexQBuffer {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let length: usize =
            stream
                .read_stream_le::<u16>()?
                .try_into()
                .map_err(|_| Error::InvalidRead {
                    message: "NexQBuffer length does not fit into usize",
                })?;

        let bytes = stream.read_byte_stream(length)?;
        Ok(ReadOutput::new(NexQBuffer(bytes), stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for NexQBuffer {
    fn get_size(&self) -> usize {
        self.0.len() + mem::size_of::<u16>()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let len: u16 = self.0.len().try_into().map_err(|_| Error::InvalidWrite {
            message: "NexQBuffer length does not fit into u16",
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

impl From<NexQBuffer> for Vec<u8> {
    fn from(nex: NexQBuffer) -> Self {
        nex.0
    }
}

impl From<Vec<u8>> for NexQBuffer {
    fn from(raw: Vec<u8>) -> Self {
        NexQBuffer(raw)
    }
}
