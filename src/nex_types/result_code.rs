use core::mem;
use no_std_io::{EndianRead, EndianWrite, Error, ReadOutput, Writer};

#[derive(Default)]
pub struct ResultCode(u32);

impl EndianRead for ResultCode {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let result = u32::try_read_le(bytes)?.into_other();
        Ok(result)
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl From<u32> for ResultCode {
    fn from(result_code: u32) -> Self {
        Self(result_code)
    }
}

impl From<ResultCode> for u32 {
    fn from(result_code: ResultCode) -> Self {
        result_code.0
    }
}

impl EndianWrite for ResultCode {
    fn get_size(&self) -> usize {
        mem::size_of::<u32>()
    }

    fn try_write_le(&self, mut dst: &mut [u8]) -> Result<usize, Error> {
        dst.write_le(0, &self.0)?;
        Ok(self.get_size())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
