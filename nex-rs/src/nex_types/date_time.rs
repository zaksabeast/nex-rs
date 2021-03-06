use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamWriter,
};
use time::OffsetDateTime;

#[derive(Debug, Default, PartialEq)]
pub struct DateTime {
    value: u64,
}

impl DateTime {
    pub fn now() -> Self {
        OffsetDateTime::now_utc().into()
    }

    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn from_time(year: u64, month: u64, day: u64, hour: u64, minute: u64, second: u64) -> Self {
        let value =
            second | (minute << 6) | (hour << 12) | (day << 17) | (month << 22) | (year << 26);
        DateTime { value }
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

impl From<DateTime> for u64 {
    fn from(datetime: DateTime) -> Self {
        datetime.value
    }
}

impl EndianRead for DateTime {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let result = u64::try_read_le(bytes)?.into_other();
        Ok(result)
    }

    fn try_read_be(_bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for DateTime {
    fn get_size(&self) -> usize {
        self.value.get_size()
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&self.value)?;
        Ok(stream.get_index())
    }

    fn try_write_be(&self, _dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}

impl From<OffsetDateTime> for DateTime {
    fn from(datetime: OffsetDateTime) -> Self {
        DateTime::from_time(
            datetime.year().try_into().unwrap_or_default(),
            {
                let month: u8 = datetime.month().into();
                month.into()
            },
            datetime.day().into(),
            datetime.hour().into(),
            datetime.minute().into(),
            datetime.second().into(),
        )
    }
}
