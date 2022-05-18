use crate::utils::enum_variant::EnumVariant;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

pub fn impl_nex_route(item: TokenStream) -> TokenStream {
    let enum_variant = parse_macro_input!(item as EnumVariant);
    let enum_ident = &enum_variant.ident;
    let variant_token = enum_variant.token();

    let protocol_id = quote! { <#enum_ident as nex_rs::route::NexProtocol>::PROTOCOL_ID as u8 };
    let method_id = quote! { #variant_token as u32 };

    quote! {
      nex_rs::route::Route::<{ #protocol_id }, { #method_id }>::run
    }
    .into()
}
