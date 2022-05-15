use proc_macro::TokenStream;

mod match_nex_route;
mod nex_method;
mod nex_route;
mod utils;

#[proc_macro_attribute]
pub fn nex_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    nex_method::impl_nex_method(attr, item)
}

#[proc_macro]
pub fn nex_route(item: TokenStream) -> TokenStream {
    nex_route::impl_nex_route(item)
}

#[proc_macro]
pub fn match_nex_route(item: TokenStream) -> TokenStream {
    match_nex_route::impl_match_nex_route(item)
}
