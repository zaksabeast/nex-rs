use no_std_io::{EndianRead, Error, ReadOutput};

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
