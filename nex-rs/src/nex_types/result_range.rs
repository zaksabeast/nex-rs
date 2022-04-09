use no_std_io::{EndianRead, EndianWrite};

#[derive(Debug, EndianRead, EndianWrite, Default)]
pub struct ResultRange {
    offset: u32,
    length: u32,
}
