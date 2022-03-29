use no_std_io::{Cursor, EndianRead, EndianWrite, Error, ReadOutput, StreamContainer, StreamReader, StreamWriter};
use crate::nex_types::{DateTime, NexString};

pub enum NexVariantType {
    Int64(i64),
    Float64(f64),
    Bool(bool),
    String(NexString),
    DateTime(DateTime),
    UInt64(u64),
}

impl NexVariantType {
    pub fn get_type_value(&self) -> u8 {
        match &self {
            NexVariantType::Int64(i) => 1,
            NexVariantType::Float64(f) => 2,
            NexVariantType::Bool(_) => 3,
            NexVariantType::String(_) => 4,
            NexVariantType::DateTime(_) => 5,
            NexVariantType::UInt64(_) => 6,
        }
    }
}

pub struct NexVariant(Option<NexVariantType>);

impl EndianRead for NexVariant {
    fn try_read_le(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        let mut stream = StreamContainer::new(bytes);
        let variant_type = stream.read_stream_le::<u8>()?;
        let data = match variant_type {
            1 => Some(NexVariantType::Int64(stream.read_stream_le::<i64>()?)),
            2 => Some(NexVariantType::Float64(stream.read_stream_le::<u64>()? as f64)),
            3 => Some(NexVariantType::Bool(stream.read_stream_le::<u8>()? == 1)),
            4 => Some(NexVariantType::String(stream.read_stream_le::<NexString>()?)),
            5 => Some(NexVariantType::DateTime(stream.read_stream_le::<DateTime>()?)),
            6 => Some(NexVariantType::UInt64(stream.read_stream_le::<u64>()?)),
            _ => None,
        };
        Ok(ReadOutput::new(NexVariant(data), stream.get_index()))
    }

    fn try_read_be(bytes: &[u8]) -> Result<ReadOutput<Self>, Error> {
        unimplemented!()
    }
}

impl EndianWrite for NexVariant {
    fn get_size(&self) -> usize {
        match self.0.as_ref().unwrap() {
            NexVariantType::Int64(i) => {
                i.get_size()
            }
            NexVariantType::Float64(_) => {
                64
            }
            NexVariantType::Bool(b) => {
                b.get_size()
            }
            NexVariantType::String(s) => {
                s.get_size()
            }
            NexVariantType::DateTime(dt) => {
                dt.get_size()
            }
            NexVariantType::UInt64(i) => {
                i.get_size()
            }
        }
    }

    fn try_write_le(&self, dst: &mut [u8]) -> Result<usize, Error> {
        let variant = self.0.as_ref().expect("No value in variant");
        let variant_type = variant.get_type_value();
        let mut stream = StreamContainer::new(dst);
        stream.write_stream_le(&variant_type)?;
        match variant {
            NexVariantType::Int64(i) => stream.write_stream_le(i)?,
            NexVariantType::Float64(f) => stream.write_stream_le(&(*f as u64))?,
            NexVariantType::Bool(b) => stream.write_stream_le(b)?,
            NexVariantType::String(s) => stream.write_stream_le(s)?,
            NexVariantType::DateTime(dt) => stream.write_stream_le(dt)?,
            NexVariantType::UInt64(i) => stream.write_stream_le(i)?,
        };
        Ok(stream.get_index())
    }

    fn try_write_be(&self, dst: &mut [u8]) -> Result<usize, Error> {
        unimplemented!()
    }
}