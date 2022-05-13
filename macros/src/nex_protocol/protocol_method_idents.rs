use heck::AsSnakeCase;
use proc_macro2::Span;
use quote::format_ident;
use syn::{Ident, Variant};

pub struct NexProtocolMethodIdentifiers {
    pub method_ident: Ident,
    pub handle_method_ident: Ident,
    pub method_name: String,
}

impl NexProtocolMethodIdentifiers {
    pub fn new(enum_variant: &Variant) -> Self {
        let method_name = AsSnakeCase(enum_variant.ident.to_string()).to_string();
        let method_ident = Ident::new(&method_name, Span::call_site());
        let handle_method_ident = format_ident!("handle_{}", method_ident);

        Self {
            method_ident,
            handle_method_ident,
            method_name,
        }
    }
}
