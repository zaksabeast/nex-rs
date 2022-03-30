use super::NexString;
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default)]
struct DataHolder<T: EndianRead + EndianWrite> {
    name: NexString,
    object: T,
}

impl<T: EndianRead + EndianWrite> DataHolder<T> {
    pub fn new_from_object(type_name: String, object: T) -> Self {
        Self {
            name: NexString::from(type_name),
            object,
        }
    }
}

impl<T: EndianRead + EndianWrite> From<T> for DataHolder<T> {
    fn from(object: T) -> Self {
        Self {
            name: NexString::default(),
            object,
        }
    }
}

impl<T: EndianRead + EndianWrite> EndianRead for DataHolder<T> {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let name = stream.read_stream_le::<NexString>()?;

        //skip total length
        let _ = stream.read_stream_le::<u32>()?;
        //skip content length
        let _ = stream.read_stream_le::<u32>()?;

        let object = stream.read_stream_le::<T>()?;

        let data_holder = Self { name, object };

        Ok(ReadOutput::new(data_holder, stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl<T: EndianRead + EndianWrite> EndianWrite for DataHolder<T> {
    fn get_size(&self) -> usize {
        self.name.get_size() + self.object.get_size() + 8
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        let total_length: u32 =
            (self.object.get_size() + 4)
                .try_into()
                .map_err(|_| Error::InvalidWrite {
                    message: "Data holder total length does not fit into u32",
                })?;
        let object_length: u32 =
            self.object
                .get_size()
                .try_into()
                .map_err(|_| Error::InvalidWrite {
                    message: "Data holder object length does not fit into u32",
                })?;

        stream.write_stream_le(&self.name)?;
        stream.write_stream_le(&total_length)?;
        stream.write_stream_le(&object_length)?;
        stream.write_stream_le(&self.object)?;

        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
