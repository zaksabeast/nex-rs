pub mod dummy_compression {
    pub fn compress(data: &[u8]) -> &[u8] {
        data
    }

    pub fn decompress(data: &[u8]) -> &[u8] {
        data
    }
}

pub mod zlib_compression {
    pub fn compress(data: &[u8]) -> &[u8] {
        data
    }

    pub fn decompress(data: &[u8]) -> &[u8] {
        data
    }
}