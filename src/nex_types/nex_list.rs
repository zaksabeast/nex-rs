use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default)]
pub struct NexList<T: EndianWrite + EndianRead>(Vec<T>);

impl<T: EndianWrite + EndianRead> From<Vec<T>> for NexList<T> {
    fn from(data: Vec<T>) -> Self {
        Self(data)
    }
}

impl<T: EndianWrite + EndianRead> From<NexList<T>> for Vec<T> {
    fn from(list: NexList<T>) -> Self {
        list.0
    }
}

impl<T: EndianWrite + EndianRead> EndianRead for NexList<T> {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let mut length: usize =
            stream
                .read_stream_le::<u32>()?
                .try_into()
                .map_err(|_| Error::InvalidRead {
                    message: "NexMap length does not fit into usize",
                })?;

        let mut data = Vec::with_capacity(length);

        while length > 0 {
            let value = stream.read_stream_le::<T>()?;
            data.push(value);
            length -= 1;
        }

        Ok(ReadOutput::new(NexList(data), stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl<T: EndianWrite + EndianRead> EndianWrite for NexList<T> {
    fn get_size(&self) -> usize {
        let mut size = 4;
        for item in &self.0 {
            size += item.get_size()
        }
        size
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let len: u32 = self.0.len().try_into().map_err(|_| Error::InvalidWrite {
            message: "NexList length does not fit into u32",
        })?;

        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&len)?;

        for entry in &self.0 {
            stream.write_stream_le(entry)?;
        }
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
