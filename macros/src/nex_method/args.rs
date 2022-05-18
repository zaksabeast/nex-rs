use crate::utils::enum_variant::EnumVariant;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Path, Result, Token,
};

#[derive(Debug)]
pub struct MethodArg {
    _path: Path,
    _equals: Token![=],
    pub variant: EnumVariant,
}

impl Parse for MethodArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let path: Path = input.parse()?;
        if !path.is_ident("method") {
            return Err(input.error("Missing 'method' argument"));
        }

        Ok(Self {
            _path: path,
            _equals: input.parse()?,
            variant: input.parse()?,
        })
    }
}

#[derive(Debug)]
enum Arg {
    Method(MethodArg),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> Result<Self> {
        let error_fork = input.fork();
        let path: Path = input.fork().parse()?;

        if path.is_ident("method") {
            return Ok(Self::Method(input.parse()?));
        }

        Err(error_fork.error("Invalid argument"))
    }
}

#[derive(Debug)]
pub struct Args {
    pub method: MethodArg,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let error_fork = input.fork();
        let args = Punctuated::<Arg, Comma>::parse_terminated(input)?;

        let mut method: Option<MethodArg> = None;

        for arg in args {
            match arg {
                Arg::Method(method_arg) => method = Some(method_arg),
            };
        }

        Ok(Self {
            method: method.ok_or_else(|| error_fork.error("'method' argument is required"))?,
        })
    }
}
