use super::NexString;
use no_std_io::{Cursor, EndianRead, EndianWrite, Error};

#[derive(Debug, EndianRead, EndianWrite)]
struct DataHolder<T: EndianRead + EndianWrite> {
    name: NexString,
    object: T,
}

impl<T: EndianRead + EndianWrite> DataHolder<T> {
    pub fn new_from_object(object: T) -> Self {
        Self {
            name: NexString::default(),
            object,
        }
    }
}
