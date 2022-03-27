use crate::nex_types::StructureInterface;
use no_std_io::{Cursor, EndianWrite, Reader, StreamContainer, StreamWriter, Writer, WriterResult};

pub struct StreamOut {
    data: StreamContainer<Vec<u8>>,
}

impl StreamOut {
    pub fn get_slice(&self) -> &[u8] {
        self.data.get_slice()
    }
}

impl Writer for StreamOut {
    fn get_mut_slice(&mut self) -> &mut [u8] {
        self.data.get_mut_slice()
    }

    fn get_sized_mut_slice(&mut self, offset: usize, length: usize) -> WriterResult<&mut [u8]> {
        self.data.get_sized_mut_slice(offset, length)
    }
}

impl Cursor for StreamOut {
    fn get_index(&self) -> usize {
        self.data.get_index()
    }

    fn set_index(&mut self, index: usize) {
        self.data.set_index(index)
    }
}

impl Default for StreamOut {
    fn default() -> Self {
        Self {
            data: StreamContainer::new(vec![]),
        }
    }
}

impl StreamOut {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn checked_write_stream_bool(&mut self, value: bool) {
        self.checked_write_stream(&u8::from(value));
    }

    pub fn write_string(&mut self, value: &str) {
        let len: u16 = (value.len() + 1)
            .try_into()
            .expect("Length does not fit into u16");
        self.checked_write_stream_le(&len);
        self.checked_write_stream_bytes(value.as_bytes());
        self.checked_write_stream(&0u8);
    }

    pub fn write_buffer(&mut self, value: &[u8]) {
        let len: u32 = value
            .len()
            .try_into()
            .expect("Length does not fit into u32");
        self.checked_write_stream_le(&len);
        self.checked_write_stream_bytes(value);
    }

    pub fn write_qbuffer(&mut self, value: &[u8]) {
        let len: u16 = (value.len() + 1)
            .try_into()
            .expect("String length does not fit into u16");
        self.checked_write_stream_le(&len);
        self.checked_write_stream_bytes(value);
    }

    pub fn write_list<Item: EndianWrite + Default>(&mut self, value: &[Item]) {
        let len: u32 = value
            .len()
            .try_into()
            .expect("Length does not fit into u32");

        self.checked_write_stream_le(&len);
        for item in value {
            self.checked_write_stream_le(item);
        }
    }

    pub fn write_string_list(&mut self, value: &[&str]) {
        let len: u32 = value
            .len()
            .try_into()
            .expect("Length does not fit into u32");

        self.checked_write_stream_le(&len);
        for item in value {
            self.write_string(item);
        }
    }

    pub fn write_qbuffer_list(&mut self, value: &[&[u8]]) {
        let len: u32 = value
            .len()
            .try_into()
            .expect("Length does not fit into u32");

        self.checked_write_stream_le(&len);
        for item in value {
            self.write_qbuffer(item);
        }
    }

    pub fn write_struct<T: StructureInterface>(&mut self, value: &T) -> Result<(), &'static str> {
        value.bytes(self)
    }
}

impl From<StreamOut> for Vec<u8> {
    fn from(stream: StreamOut) -> Self {
        stream.data.into_raw()
    }
}

impl From<Vec<u8>> for StreamOut {
    fn from(data: Vec<u8>) -> Self {
        let len = data.len();
        let mut container = StreamContainer::new(data);
        container.set_index(len);

        Self { data: container }
    }
}
