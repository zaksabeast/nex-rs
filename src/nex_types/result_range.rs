use no_std_io::{Cursor, EndianRead};

#[derive(Debug, EndianRead, Default)]
pub struct ResultRange {
    offset: u32,
    length: u32,
}
