use crate::nex_types::StructureInterface;
use no_std_io::{Cursor, EndianRead, Reader, StreamContainer, StreamReader};

pub struct StreamIn<T: Reader> {
    data: StreamContainer<T>,
}

impl<T: Reader> Reader for StreamIn<T> {
    fn get_slice(&self) -> &[u8] {
        self.data.get_slice()
    }
}

impl<T: Reader> Cursor for StreamIn<T> {
    fn get_index(&self) -> usize {
        self.data.get_index()
    }

    fn set_index(&mut self, index: usize) {
        self.data.set_index(index)
    }
}

impl<T: Reader> StreamIn<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: StreamContainer::new(data),
        }
    }

    pub fn default_read_steam_bool(&mut self) -> bool {
        self.default_read_stream::<u8>() != 0
    }

    pub fn read_string(&mut self) -> String {
        let length: u16 = self.default_read_stream_le();
        let bytes = self.default_read_byte_stream(length.into());
        String::from_utf8(bytes).unwrap_or_default()
    }

    pub fn read_buffer(&mut self) -> Vec<u8> {
        let length: u32 = self.default_read_stream_le();
        self.default_read_byte_stream(length.try_into().expect("Invalid buffer size"))
    }

    pub fn read_qbuffer(&mut self) -> Vec<u8> {
        let length: u16 = self.default_read_stream_le();
        self.default_read_byte_stream(length.into())
    }

    pub fn read_list<Item: EndianRead + Default>(&mut self) -> Vec<Item> {
        let length: u32 = self.default_read_stream_le();
        let mut list = Vec::with_capacity(length.try_into().expect("Invalid buffer size"));

        for _ in 0..length {
            list.push(self.default_read_stream_le());
        }

        list
    }

    pub fn read_string_list(&mut self) -> Vec<String> {
        let length: u32 = self.default_read_stream_le();
        let mut list = vec![];

        for _ in 0..length {
            list.push(self.read_string());
        }

        list
    }

    pub fn read_qbuffer_list(&mut self) -> Vec<Vec<u8>> {
        let length: u32 = self.default_read_stream_le();
        let mut list = vec![];

        for _ in 0..length {
            list.push(self.read_qbuffer());
        }

        list
    }

    pub fn read_struct<S: StructureInterface>(&mut self) -> Result<S, &'static str> {
        S::extract_from_stream(self)
    }
}
