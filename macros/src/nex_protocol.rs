use super::protocol_method::ProtocolMethodArgs;
use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{self, parse_macro_input, Data, DataEnum, DeriveInput, Ident, Variant};

struct NexProtocolIdentifiers {
    protocol_ident: Ident,
    protocol_name: String,
    protocol_method_name: String,
}

impl NexProtocolIdentifiers {
    fn new(enum_ident: &Ident) -> Self {
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

struct NexProtocolMethodIdentifiers {
    method_ident: Ident,
    handle_method_ident: Ident,
    method_name: String,
}

impl NexProtocolMethodIdentifiers {
    fn new(enum_variant: &Variant) -> Self {
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

fn get_variant_method(variant: &Variant) -> Option<proc_macro2::TokenStream> {
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
    let protocol_method_ident = &input.ident;

    let enum_variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("Only enums can derive NexProtocol"),
    };

    let variant_methods = enum_variants
        .iter()
        .map(|variant| get_variant_method(variant).unwrap_or_default())
        .collect::<Vec<proc_macro2::TokenStream>>();

    let variant_handle_branches = enum_variants
        .iter()
        .map(|variant| get_variant_handle_branch(protocol_method_ident, variant))
        .collect::<Vec<proc_macro2::TokenStream>>();

    let NexProtocolIdentifiers {
        protocol_ident,
        protocol_name,
        protocol_method_name,
    } = NexProtocolIdentifiers::new(protocol_method_ident);
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
                let parsed_method = #protocol_method_ident::try_from(request.method_id).ok();

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
