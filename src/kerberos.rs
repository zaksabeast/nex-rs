use hmac::{Hmac, Mac};
use md5::Md5;
use crate::rc4::Rc4;

struct KerberosEncryption {
    key: Vec<u8>,
    cipher: Rc4
}

impl KerberosEncryption {
    pub fn new(key: Vec<u8>) -> Self {
        KerberosEncryption {
            key: key.clone(),
            cipher: Rc4::new(&key)
        }
    }

    pub fn encrypt(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, &'static str> {
        let mut encrypted = self.cipher.encrypt(&buffer).expect("Encrypt failed");

        let mut mac = Hmac::<Md5>::new_from_slice(&self.key).map_err(|_| "Invalid hamc key size")?;
        mac.update(&encrypted);

        encrypted.append(&mut mac.finalize().into_bytes().to_vec());
        Ok(encrypted)
    }

    pub fn decrypt(&mut self, buffer: Vec<u8>) -> Result<Vec<u8>, &'static str> {
        if self.validate(buffer.clone())? {
            let offset = buffer.len() - 0x10;
            let encrypted = &buffer[..offset];

            self.cipher.decrypt(encrypted)
        } else {
            Err("INVALID KERB CHECKSUM")
        }
    }

    pub fn validate(&self, buffer: Vec<u8>) -> Result<bool, &'static str> {
        let offset = buffer.len() - 0x10;
        let data = &buffer[..offset];
        let checksum = &buffer[offset..];

        let mut cipher = Hmac::<Md5>::new_from_slice(&self.key).map_err(|_| "Invalid hamc key size")?;
        cipher.update(data);

        let mac = cipher.finalize().into_bytes().to_vec();
        Ok(mac.as_slice() == checksum)
    }
}


struct Ticket {
    session_key: Vec<u8>,
    server_pid: u32,
    ticket_data: Vec<u8>
}

struct TicketData {
    ticket_key: Vec<u8>,
    ticket_info: Vec<u8>
}

struct TicketInfo {
    datetime: u64,
    user_pid: u32,
    session_key: Vec<u8>
}

