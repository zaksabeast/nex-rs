use crypto::{
    buffer::{RefReadBuffer, RefWriteBuffer},
    rc4,
    symmetriccipher::{Decryptor, Encryptor},
};

#[derive(Clone)]
pub struct Rc4 {
    inner: rc4::Rc4,
}

impl Rc4 {
    pub fn new(key: &[u8]) -> Self {
        Self {
            inner: rc4::Rc4::new(key),
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let mut out = vec![0; data.len()];
        let mut out_buf = RefWriteBuffer::new(&mut out);
        let mut in_buf = RefReadBuffer::new(data);
        self.inner
            .encrypt(&mut in_buf, &mut out_buf, true)
            .map_err(|_| "Encrypt failed")?;

        Ok(out)
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        let mut out = vec![0; data.len()];
        let mut out_buf = RefWriteBuffer::new(&mut out);
        let mut in_buf = RefReadBuffer::new(data);
        self.inner
            .decrypt(&mut in_buf, &mut out_buf, true)
            .map_err(|_| "Decrypt failed")?;

        Ok(out)
    }
}
