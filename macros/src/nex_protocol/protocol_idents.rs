use heck::AsSnakeCase;
use quote::format_ident;
use syn::Ident;

pub struct NexProtocolIdentifiers {
    pub protocol_ident: Ident,
    pub protocol_name: String,
    pub protocol_method_name: String,
}

impl NexProtocolIdentifiers {
    pub fn new(enum_ident: &Ident) -> Self {
        let protocol_method_name = enum_ident.to_string();
        let protocol_name = protocol_method_name.replace("Method", "");
        let protocol_ident = format_ident!("{}Protocol", protocol_name);

        Self {
            protocol_ident,
            protocol_name: AsSnakeCase(protocol_name).to_string(),
            protocol_method_name,
        }
    }
}
