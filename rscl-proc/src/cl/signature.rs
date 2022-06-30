use proc_macro2::Ident;
use syn::{parse::Parse, custom_keyword, Token, punctuated::Punctuated, parenthesized};
use super::arg::FnArg;

custom_keyword!(kernel);

pub struct Signature {
    pub kernel_token: kernel,
    pub fn_token: Token![fn],
    pub ident: Ident,
    pub paren_token: syn::token::Paren,
    pub inputs: Punctuated<FnArg, Token![,]>
}

impl Parse for Signature {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let kernel_token = input.parse()?;
        let fn_token = input.parse()?;
        let ident = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);
        let inputs = content.parse_terminated(FnArg::parse)?;
        
        Ok(Self { kernel_token, fn_token, ident, paren_token, inputs })
    }
}