use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamWriter,
};

pub struct DateTime {
    value: u64,
}

impl DateTime {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn make(
        &mut self,
        year: u64,
        month: u64,
        day: u64,
        hour: u64,
        minute: u64,
        second: u64,
    ) -> u64 {
        self.value =
            second | (minute << 6) | (hour << 12) | (day << 17) | (month << 22) | (year << 26);
        self.value
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }
}

impl From<u64> for DateTime {
    fn from(raw: u64) -> Self {
        Self::new(raw)
    }
}

impl EndianRead for DateTime {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let result = u64::try_read_le(bytes)?.into_other();
        Ok(result)
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for DateTime {
    fn get_size(&self) -> usize {
        64
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.value)?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
