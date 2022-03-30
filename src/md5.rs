use md5::{Digest, Md5};

pub fn hash(data: &[u8]) -> [u8; 16] {
    let mut md5 = Md5::new();
    md5.update(data);
    md5.finalize().try_into().unwrap()
}
