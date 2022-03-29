use crate::nex_types::{DateTime, NexString};
use no_std_io::{
    Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter,
};

pub enum NexVariant {
    Int64(i64),
    Float64(f64),
    Bool(bool),
    String(NexString),
    DateTime(DateTime),
    UInt64(u64),
    Null,
}

impl Default for NexVariant {
    fn default() -> Self {
        NexVariant::Null
    }
}

impl NexVariant {
    pub fn get_type_value(&self) -> u8 {
        match &self {
            NexVariant::Null => 0,
            NexVariant::Int64(i) => 1,
            NexVariant::Float64(f) => 2,
            NexVariant::Bool(_) => 3,
            NexVariant::String(_) => 4,
            NexVariant::DateTime(_) => 5,
            NexVariant::UInt64(_) => 6,
        }
    }
}

impl EndianRead for NexVariant {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let variant_type = stream.read_stream_le::<u8>()?;
        let data = match variant_type {
            1 => NexVariant::Int64(stream.read_stream_le::<i64>()?),
            2 => NexVariant::Float64(stream.read_stream_le::<u64>()? as f64),
            3 => NexVariant::Bool(stream.read_stream_le::<u8>()? == 1),
            4 => NexVariant::String(stream.read_stream_le::<NexString>()?),
            5 => NexVariant::DateTime(stream.read_stream_le::<DateTime>()?),
            6 => NexVariant::UInt64(stream.read_stream_le::<u64>()?),
            _ => NexVariant::Null,
        };
        Ok(ReadOutput::new(data, stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for NexVariant {
    fn get_size(&self) -> usize {
        match &self {
            NexVariant::Int64(i) => i.get_size(),
            NexVariant::Float64(_) => 16,
            NexVariant::Bool(b) => b.get_size(),
            NexVariant::String(s) => s.get_size(),
            NexVariant::DateTime(dt) => dt.get_size(),
            NexVariant::UInt64(i) => i.get_size(),
            NexVariant::Null => 0,
        }
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let variant_type = self.get_type_value();
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&variant_type)?;
        match self {
            NexVariant::Int64(i) => stream.write_stream_le(i)?,
            NexVariant::Float64(f) => stream.write_stream_le(&(*f as u64))?,
            NexVariant::Bool(b) => stream.write_stream_le(b)?,
            NexVariant::String(s) => stream.write_stream_le(s)?,
            NexVariant::DateTime(dt) => stream.write_stream_le(dt)?,
            NexVariant::UInt64(i) => stream.write_stream_le(i)?,
            NexVariant::Null => 0,
        };
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}
