#[derive(Default)]
pub struct RMCRequest {
    protocol_id: u8,
    call_id: u32,
    method_id: u32,
    parameters: Vec<u8>,
}
