use super::args::Args;
use crate::utils::enum_variant::EnumVariant;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

fn get_variant_match_branch(args: &Args, variant: &EnumVariant) -> proc_macro2::TokenStream {
    let Args {
        server,
        client,
        request,
        ..
    } = args;
    let variant_token = variant.token();

    quote! {
      #request if #request.is_method(#variant_token) => nex_rs::macros::nex_route![#variant_token](#server, #client, #request).await,
    }
}

pub fn impl_match_nex_route(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as Args);
    let request = &args.request;

    let match_branches = args
        .variants
        .iter()
        .map(|variant| get_variant_match_branch(&args, variant))
        .collect::<Vec<proc_macro2::TokenStream>>();

    quote! {
      match #request {
        #(#match_branches)*
        _ => Ok(())
      }
    }
    .into()
}
