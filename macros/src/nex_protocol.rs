use super::protocol_method::ProtocolMethodArgs;
use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DataEnum, DeriveInput, Ident, Variant};

fn create_variant_method(variant: &Variant) -> Option<proc_macro2::TokenStream> {
    let method_name = AsSnakeCase(variant.ident.to_string()).to_string();
    let method_ident = Ident::new(&method_name, Span::call_site());
    let handle_method_ident = format_ident!("handle_{}", method_ident);
    let input_read_error = format!("Cannot read {} input", method_name);

    let ProtocolMethodArgs { input, output } = ProtocolMethodArgs::new(variant)?;

    let method_input = match input.clone() {
        Some(input) => quote!(input: #input),
        None => quote!(),
    };

    let input_read = match input.clone() {
        Some(input) => quote!(
            no_std_io::StreamReader::read_stream_le::<#input>(&mut parameters_stream)
                .map_err(|_| #input_read_error)?
        ),
        None => quote!(()),
    };

    let input_use = match input {
        Some(_) => quote!(param),
        None => quote!(),
    };

    let method_output = match output.clone() {
        Some(output) => quote!(#output),
        None => quote!(()),
    };

    let output_write = match output {
        Some(_) => quote!(no_std_io::Writer::checked_write_le(&mut data, 0, &response);),
        None => quote!(),
    };

    let tokens = quote!(
        async fn #method_ident(
            &self,
            client: &mut nex_rs::client::ClientConnection,
            #method_input
        ) -> Result<#method_output, nex_rs::nex_types::ResultCode>;

        async fn #handle_method_ident(
            &self,
            client: &mut nex_rs::client::ClientConnection,
            request: &nex_rs::rmc::RMCRequest,
        ) -> nex_rs::result::NexResult<()> {
            let parameters = request.parameters.as_slice();
            let mut parameters_stream = no_std_io::StreamContainer::new(parameters);

            let param = #input_read;

            match Self::#method_ident(self, client, #input_use).await {
                Ok(response) => {
                   let mut data = vec![];
                   #output_write
                   nex_rs::server::Server::send_success(
                        self,
                        client,
                        request.protocol_id,
                        request.method_id,
                        request.call_id,
                        data,
                    )
                    .await?
                }
                Err(error_code) => {
                    nex_rs::server::Server::send_error(
                        self,
                        client,
                        request.protocol_id,
                        request.method_id,
                        request.call_id,
                        error_code.into(),
                    )
                    .await?
                }
            }
            Ok(())
        }
    );

    Some(tokens)
}

pub fn impl_nex_protocol(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);

    let enum_variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Only enums can derive NexProtocol"),
    };

    let variant_methods = enum_variants
        .iter()
        .map(|variant| create_variant_method(variant).unwrap_or_default())
        .collect::<Vec<proc_macro2::TokenStream>>();

    let protocol_name = input.ident.to_string().replace("Method", "");
    let protocol_ident = Ident::new(&format!("{}Protocol", protocol_name), Span::call_site());

    quote! {
        #[async_trait::async_trait]
        pub trait #protocol_ident: nex_rs::server::Server {
            #(#variant_methods)*
        }
    }
    .into()
}
