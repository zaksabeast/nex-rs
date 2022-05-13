use super::{
    protocol_idents::NexProtocolIdentifiers, protocol_method::ProtocolMethodArgs,
    protocol_method_idents::NexProtocolMethodIdentifiers,
};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Ident, Variant};

fn get_variant_methods(variant: &Variant) -> Option<proc_macro2::TokenStream> {
    let NexProtocolMethodIdentifiers {
        method_name,
        method_ident,
        handle_method_ident,
        ..
    } = NexProtocolMethodIdentifiers::new(variant);
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
        ) -> Result<#method_output, Self::Error>;

        async fn #handle_method_ident(
            &self,
            client: &mut nex_rs::client::ClientConnection,
            request: &nex_rs::rmc::RMCRequest,
        ) -> nex_rs::server::ServerResult<()> {
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
                Err(error) => {
                    let error_code = nex_rs::result::NexError::error_code(&error);
                    nex_rs::server::EventHandler::on_error(self, &error.into()).await;
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

fn get_variant_handle_branch(enum_ident: &Ident, variant: &Variant) -> proc_macro2::TokenStream {
    let NexProtocolIdentifiers { protocol_ident, .. } = NexProtocolIdentifiers::new(enum_ident);
    let NexProtocolMethodIdentifiers {
        handle_method_ident,
        ..
    } = NexProtocolMethodIdentifiers::new(variant);
    let variant_ident = &variant.ident;

    quote! {
        #enum_ident::#variant_ident => #protocol_ident::#handle_method_ident(self, client, request).await?,
    }
}

pub fn impl_nex_protocol(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);

    let protocol_method_variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Only enums can derive NexProtocol"),
    };

    let protocol_method_enum_ident = &input.ident;

    let variant_methods = protocol_method_variants
        .iter()
        .map(|variant| get_variant_methods(variant).unwrap_or_default())
        .collect::<Vec<proc_macro2::TokenStream>>();

    let variant_handle_branches = protocol_method_variants
        .iter()
        .map(|variant| get_variant_handle_branch(protocol_method_enum_ident, variant))
        .collect::<Vec<proc_macro2::TokenStream>>();

    let NexProtocolIdentifiers {
        protocol_ident,
        protocol_name,
        protocol_method_name,
    } = NexProtocolIdentifiers::new(protocol_method_enum_ident);
    let handle_method = format_ident!("handle_{}_method", protocol_name);

    quote! {
        #[async_trait::async_trait]
        pub trait #protocol_ident: nex_rs::server::Server {
            type Error: nex_rs::result::NexError;
            #(#variant_methods)*

            async fn #handle_method(
                &self,
                client: &mut nex_rs::client::ClientConnection,
                request: &nex_rs::rmc::RMCRequest,
            ) -> nex_rs::server::ServerResult<()> {
                let parsed_method = #protocol_method_enum_ident::try_from(request.method_id).ok();

                if let Some(method) = parsed_method {
                    nex_rs::server::EventHandler::on_protocol_method(self, format!("{}::{:?}", #protocol_method_name, method)).await;
                    match method {
                        #(#variant_handle_branches)*
                    }
                };

                Ok(())
            }
        }
    }
    .into()
}
