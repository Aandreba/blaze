use derive_syn_parse::Parse;
use proc_macro2::Ident;
use syn::{custom_keyword, Token, punctuated::Punctuated, Visibility};
use super::arg::FnArg;

custom_keyword!(kernel);

#[derive(Parse)]
#[non_exhaustive]
pub struct Signature {
    pub vis: Visibility,
    pub kernel_token: kernel,
    pub fn_token: Token![fn],
    pub ident: Ident,
    #[paren]
    pub paren_token: syn::token::Paren,
    #[inside(paren_token)]
    #[call(Punctuated::parse_terminated)]
    pub inputs: Punctuated<FnArg, Token![,]>
}