use proc_macro2::Span;
use syn::{
    self, punctuated::Punctuated, token::Comma, Attribute, Ident, Lit, Meta, MetaNameValue, Variant,
};

#[derive(Debug)]
pub struct ProtocolMethodArgs {
    pub input: Option<Ident>,
    pub output: Option<Ident>,
}

fn get_str_ident(meta: &MetaNameValue, ident: &str) -> Option<Ident> {
    if meta.path.is_ident(ident) {
        if let Lit::Str(value) = &meta.lit {
            let ident = Ident::new(&value.value(), Span::call_site());
            return Some(ident);
        }
    }

    None
}

impl ProtocolMethodArgs {
    pub fn new(variant: &Variant) -> Option<ProtocolMethodArgs> {
        variant.attrs.iter().find_map(|attr| {
            if attr.path.is_ident("protocol_method") {
                return Some(ProtocolMethodArgs::from_attr(attr));
            }

            None
        })
    }

    fn from_attr(attr: &Attribute) -> Self {
        let metas = attr
            .parse_args_with(Punctuated::<Meta, Comma>::parse_terminated)
            .unwrap_or_default();

        let mut result = Self {
            input: None,
            output: None,
        };

        for meta in metas {
            match meta {
                Meta::NameValue(name_value) => {
                    if let Some(ident) = get_str_ident(&name_value, "input") {
                        result.input = Some(ident);
                    }

                    if let Some(ident) = get_str_ident(&name_value, "output") {
                        result.output = Some(ident);
                    }
                }
                _ => panic!("Found invalid argument"),
            };
        }

        result
    }
}
