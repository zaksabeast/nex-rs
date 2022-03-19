use crate::server::Server;
use no_std_io::{Cursor, EndianRead, Reader, StreamContainer, StreamReader};

pub struct StreamIn<'a> {
    data: StreamContainer<Vec<u8>>,
    server: &'a Server,
}

impl<'a> Reader for StreamIn<'a> {
    fn get_slice(&self) -> &[u8] {
        self.data.get_slice()
    }
}

impl<'a> Cursor for StreamIn<'a> {
    fn get_index(&self) -> usize {
        self.data.get_index()
    }

    fn set_index(&mut self, index: usize) {
        self.data.set_index(index)
    }
}

impl<'a> StreamIn<'a> {
    pub fn new(data: Vec<u8>, server: &'a Server) -> Self {
        Self {
            data: StreamContainer::new(data),
            server,
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

    pub fn read_list<T: EndianRead + Default>(&mut self) -> Vec<T> {
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
}
