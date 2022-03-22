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
            cipher: Rc4::new(&key),
            key
        }
    }

    pub fn encrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, &'static str> {
        let mut encrypted = self.cipher.encrypt(buffer).expect("Encrypt failed");

        let mut mac = Hmac::<Md5>::new_from_slice(&self.key).map_err(|_| "Invalid hamc key size")?;
        mac.update(&encrypted);

        encrypted.append(&mut mac.finalize().into_bytes().to_vec());
        Ok(encrypted)
    }

    pub fn decrypt(&mut self, buffer: &[u8]) -> Result<Vec<u8>, &'static str> {
        if self.validate(buffer)? {
            let offset = buffer.len() - 0x10;
            let encrypted = &buffer[..offset];

            self.cipher.decrypt(encrypted)
        } else {
            Err("INVALID KERB CHECKSUM")
        }
    }

    pub fn validate(&self, buffer: &[u8]) -> Result<bool, &'static str> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_encrypt_and_decrypt() {
        let data = "kerberos test".as_bytes();
        let key = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F];
        let mut kerb = KerberosEncryption::new(key.clone());
        let encrypted = kerb.encrypt(data).expect("Failed to encrypt");
        kerb = KerberosEncryption::new(key);
        let decrypted = kerb.decrypt(encrypted.as_slice()).expect("Failed to decrypt");
        assert_eq!(data, decrypted);
    }

    #[test]
    fn should_encrypt() {
        let data = "encrypt me".as_bytes();
        let result = vec![0xbb, 0x76, 0xea, 0x33, 0xda, 0x47, 0x29, 0x1a, 0xe7, 0x63, 0xef, 0xda, 0xaa, 0xb8, 0x27, 0xe0, 0x48, 0x4c, 0x68, 0xe0, 0xf7, 0x93, 0x62, 0x27, 0x48, 0x70];
        let key = vec![0];
        let mut kerb = KerberosEncryption::new(key);
        let encrypted = kerb.encrypt(data).expect("Failed to encrypt");
        assert_eq!(encrypted, result);
    }

    #[test]
    fn should_decrypt() {
        let result = "decrypt me".as_bytes();
        let data = vec![0xba, 0x7d, 0xea, 0x33, 0xda, 0x47, 0x29, 0x1a, 0xe7, 0x63, 0x42, 0xc4, 0xe2, 0xdf, 0x3c, 0x0d, 0x5a, 0xad, 0xea, 0x22, 0xa2, 0x60, 0xd0, 0x2a, 0xb3, 0x50];
        let key = vec![0];
        let mut kerb = KerberosEncryption::new(key);
        let decrypted = kerb.decrypt(&data).expect("Failed to decrypt");
        assert_eq!(decrypted, result);
    }
}

