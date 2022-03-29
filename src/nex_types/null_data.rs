use no_std_io::{EndianRead, Error, ReadOutput};

#[derive(Default)]
pub struct NullData;

impl NullData {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EndianRead for NullData {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        Ok(ReadOutput::new(Self, 0))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}
