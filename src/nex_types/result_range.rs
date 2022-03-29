use no_std_io::EndianRead;

#[derive(Debug, EndianRead, Default)]
pub struct ResultRange {
    offset: u32,
    length: u32,
}
