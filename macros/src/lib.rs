use proc_macro::TokenStream;

mod nex_protocol;

#[proc_macro_derive(NexProtocol, attributes(protocol_method))]
pub fn impl_nex_protocol(tokens: TokenStream) -> TokenStream {
    nex_protocol::impl_nex_protocol(tokens)
}
