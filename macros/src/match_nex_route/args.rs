use crate::utils::enum_variant::EnumVariant;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Ident, Result, Token,
};

pub type EnumVariants = Punctuated<EnumVariant, Comma>;

#[derive(Debug)]
pub struct Args {
    pub server: Ident,
    _comma_1: Token![,],
    pub client: Ident,
    _comma_2: Token![,],
    pub request: Ident,
    _comma_3: Token![,],
    pub variants: EnumVariants,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            server: input.parse()?,
            _comma_1: input.parse()?,
            client: input.parse()?,
            _comma_2: input.parse()?,
            request: input.parse()?,
            _comma_3: input.parse()?,
            variants: EnumVariants::parse_terminated(input)?,
        })
    }
}
