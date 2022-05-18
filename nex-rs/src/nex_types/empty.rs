use no_std_io::{EndianRead, EndianWrite, Error, ReadOutput};

#[derive(Debug, Default, PartialEq)]
pub struct Empty;

impl EndianRead for Empty {
    fn try_read_le(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        Ok(ReadOutput::new(Empty, 0))
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for Empty {
    fn get_size(&self) -> usize {
        0
    }

    fn try_write_le(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        Ok(0)
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
