use core::mem;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader,
    StreamWriter, Writer,
};

pub struct NexStruct<T: EndianRead + EndianWrite> {
    raw: T,
    version: u8,
}

impl<T: EndianRead + EndianWrite> NexStruct<T> {
    pub fn new(raw: T, version: u8) -> Self {
        Self { raw, version }
    }
}

impl<T: EndianRead + EndianWrite> TryFrom<NexStruct<T>> for Vec<u8> {
    type Error = no_std_io::Error;

    fn try_from(nex_struct: NexStruct<T>) -> Result<Self, Self::Error> {
        let mut result = vec![];
        result.write_le(0, &nex_struct)?;
        Ok(result)
    }
}

impl<T: EndianRead + EndianWrite> EndianRead for NexStruct<T> {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let version: u8 = stream.read_stream_le()?;
        let size: usize =
            stream
                .read_stream_le::<u32>()?
                .try_into()
                .map_err(|_| Error::InvalidRead {
                    message: "NexStruct size does not fit into usize",
                })?;

        let raw: T = stream.read_stream_le()?;

        if raw.get_size() != size {
            return Err(Error::InvalidRead {
                message: "NexStruct data size did not match read size",
            });
        }

        let nex_struct = NexStruct::new(raw, version);

        Ok(ReadOutput::new(nex_struct, stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl<T: EndianRead + EndianWrite> EndianWrite for NexStruct<T> {
    fn get_size(&self) -> usize {
        mem::size_of::<u8>() + // version 
        mem::size_of::<u32>() + // size
        self.raw.get_size()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let size: u32 = self
            .raw
            .get_size()
            .try_into()
            .map_err(|_| Error::InvalidRead {
                message: "NexStruct size does not fit into u32",
            })?;

        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.version)?;
        stream.write_stream_le(&size)?;
        stream.write_stream_le(&self.raw)?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
