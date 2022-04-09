use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct NexString(String);

impl From<NexString> for String {
    fn from(nex: NexString) -> Self {
        nex.0
    }
}

impl From<String> for NexString {
    fn from(raw: String) -> Self {
        NexString(raw)
    }
}

impl From<&str> for NexString {
    fn from(raw: &str) -> Self {
        NexString(raw.to_string())
    }
}

impl EndianWrite for NexString {
    fn get_size(&self) -> usize {
        self.0.len() + 3
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let raw = &self.0;
        let len: u16 = (self.0.len() + 1)
            .try_into()
            .map_err(|_| Error::InvalidWrite {
                message: "String length does not fit into u16",
            })?;

        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&len)?;
        stream.write_stream_bytes(raw.as_bytes())?;
        stream.write_stream(&0u8)?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl EndianRead for NexString {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let length: usize = stream.read_stream_le::<u16>()?.into();
        let content_len = length.saturating_sub(1);

        let read_bytes = stream.read_byte_stream(content_len)?;
        let null_char: u8 = stream.read_stream_le()?;

        if null_char != 0 {
            return Err(Error::InvalidRead {
                message: "Null terminator was not found!",
            });
        }

        let raw = String::from_utf8(read_bytes).map_err(|_| Error::InvalidRead {
            message: "Bytes weren't valid utf8",
        })?;

        Ok(ReadOutput::new(NexString(raw), stream.get_index()))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod try_read_le {
        use super::*;
        use no_std_io::Reader;

        #[test]
        fn should_read() {
            let bytes = vec![
                0x0b, 0x00, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x00,
            ];
            let result: NexString = bytes.read_le(0).unwrap();
            assert_eq!(result, NexString("0123456789".to_string()));
        }

        #[test]
        fn should_error_if_last_char_is_not_null() {
            let bytes = vec![
                0x0b, 0x00, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x39,
            ];
            let result = bytes.read_le::<NexString>(0).unwrap_err();
            assert_eq!(
                result,
                Error::InvalidRead {
                    message: "Null terminator was not found!"
                }
            );
        }

        #[test]
        fn should_error_if_length_is_0() {
            let bytes = vec![0x00, 0x00];
            let result = bytes.read_le::<NexString>(0).unwrap_err();
            assert_eq!(
                result,
                Error::InvalidSize {
                    wanted_size: 1,
                    offset: 2,
                    data_len: 2
                }
            );
        }
    }

    mod try_write_le {
        use super::*;
        use no_std_io::Writer;

        #[test]
        fn should_write() {
            let nex_string: NexString = "0123456789".into();

            let mut bytes = vec![];
            bytes.write_le(0, &nex_string).unwrap();

            let expected = vec![
                0x0b, 0x00, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x00,
            ];

            assert_eq!(bytes, expected);
        }
    }
}
