pub mod dummy_compression {
    pub fn compress(data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
}

pub mod zlib_compression {
    use miniz_oxide::deflate::compress_to_vec_zlib;

    pub fn compress(data: &[u8]) -> Vec<u8> {
        compress_to_vec_zlib(data, 6)
    }
}
