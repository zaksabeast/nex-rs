pub mod dummy_compression {
    pub fn compress(data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    pub fn decompress(data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
}

pub mod zlib_compression {
    use miniz_oxide::deflate::compress_to_vec_zlib;
    use miniz_oxide::inflate::{decompress_to_vec_zlib, TINFLStatus};

    pub fn compress(data: &[u8]) -> Vec<u8> {
        compress_to_vec_zlib(data, 6)
    }

    pub fn decompress(data: &[u8]) -> Result<Vec<u8>, TINFLStatus> {
        Ok(decompress_to_vec_zlib(data)?)
    }
}