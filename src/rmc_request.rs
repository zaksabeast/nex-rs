use crate::stream::StreamIn;
use no_std_io::StreamReader;

#[derive(Default, Debug)]
pub struct RMCRequest {
    protocol_id: u8,
    call_id: u32,
    method_id: u32,
    parameters: Vec<u8>,
}

impl TryFrom<&[u8]> for RMCRequest {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let bytes_len = bytes.len();
        if bytes_len < 13 {
            return Err("Invalid RMCRequest size");
        }

        let mut stream = StreamIn::new(bytes);

        let size: usize = stream
            .read_stream_le::<u32>()
            .map_err(|_| "RMCRequest size could not be read")?
            .try_into()
            .map_err(|_| "RMCRequest size does not fit into usize")?;

        if size != bytes_len - 4 {
            return Err("RMCRequest data size does not match");
        }

        Ok(Self {
            protocol_id: stream.default_read_stream(),
            call_id: stream.default_read_stream(),
            method_id: stream.default_read_stream(),
            parameters: stream.default_read_byte_stream(bytes_len - 13),
        })
    }
}
