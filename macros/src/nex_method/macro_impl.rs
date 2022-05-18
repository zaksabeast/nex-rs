use super::{
    args::{Args, MethodArg},
    method_signature::MethodSignature,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

fn get_server_impl(
    protocol_method_fn_def: &ItemFn,
    method: &MethodArg,
) -> proc_macro2::TokenStream {
    let protocol_method_fn_ident = &protocol_method_fn_def.sig.ident;
    let method_ident = method.variant.token();
    let protocol_ident = &method.variant.ident;
    let input_read_error = format!("Cannot read {} input", method.variant);

    let MethodSignature { server, input } = MethodSignature::new(protocol_method_fn_def);

    let input_read = match input.clone() {
        Some(input) => quote! {
            no_std_io::StreamReader::read_stream_le::<#input>(&mut parameters_stream)
                .map_err(|_| #input_read_error)?
        },
        None => quote!(()),
    };

    let input_use = match input {
        Some(_) => quote! {param},
        None => quote!(),
    };

    quote! {
        #[async_trait::async_trait]
        impl
            nex_rs::route::Route<
                { <#protocol_ident as nex_rs::route::NexProtocol>::PROTOCOL_ID as u8 },
                { #method_ident as u32 },
            > for #server
        {
            async fn run(
                &self,
                client: &mut nex_rs::client::ClientConnection,
                request: &nex_rs::rmc::RMCRequest,
            ) -> nex_rs::server::ServerResult<()> {
                let parameters = request.parameters.as_slice();
                let mut parameters_stream = no_std_io::StreamContainer::new(parameters);

                let param = #input_read;

                match #protocol_method_fn_ident(self, client, #input_use).await {
                    Ok(response) => {
                        let mut data = vec![];
                        no_std_io::Writer::checked_write_le(&mut data, 0, &response);
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
        }
    }
}

pub fn impl_nex_method(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as Args);
    let protocol_method_fn_def = parse_macro_input!(item as ItemFn);

    let server_impl = get_server_impl(&protocol_method_fn_def, &args.method);

    quote! {
        #protocol_method_fn_def

        #server_impl
    }
    .into()
}
