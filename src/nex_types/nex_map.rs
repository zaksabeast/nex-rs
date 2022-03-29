use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default)]
pub struct NexMap<T: EndianWrite + EndianRead, U: EndianWrite + EndianRead>(Vec<(T, U)>);

impl<T: EndianWrite + EndianRead, U: EndianWrite + EndianRead> From<Vec<(T, U)>> for NexMap<T, U> {
    fn from(map: Vec<(T, U)>) -> Self {
        Self(map)
    }
}

impl<T: EndianWrite + EndianRead, U: EndianWrite + EndianRead> EndianRead for NexMap<T, U> {
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
            let key = stream.read_stream_le::<T>()?;
            let value = stream.read_stream_le::<U>()?;
            data.push((key, value));
            length -= 1;
        }

        Ok(ReadOutput::new(NexMap(data), stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl<T: EndianWrite + EndianRead, U: EndianWrite + EndianRead> EndianWrite for NexMap<T, U> {
    fn get_size(&self) -> usize {
        let mut size = 4;
        for entry in &self.0 {
            size += entry.0.get_size();
            size += entry.1.get_size();
        }
        size
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let len: u32 = self.0.len().try_into().map_err(|_| Error::InvalidWrite {
            message: "NexMap length does not fit into u32",
        })?;

        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&len)?;

        for entry in &self.0 {
            stream.write_stream_le(&entry.0)?;
            stream.write_stream_le(&entry.1)?;
        }
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
