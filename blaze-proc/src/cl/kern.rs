use derive_syn_parse::Parse;
use proc_macro2::{Ident};
use syn::{punctuated::Punctuated, Token, LitStr, Visibility, Attribute, parse_quote_spanned, spanned::Spanned};
use super::{Argument};

#[derive(Debug, Parse)]
pub struct Kernel {
    pub attrs: KernelAttrs,
    pub vis: Visibility,
    pub fn_token: Token![fn],
    pub ident: Ident,
    #[paren]
    pub paren_token: syn::token::Paren,
    #[inside(paren_token)]
    #[call(Punctuated::parse_terminated)]
    pub args: Punctuated<Argument, Token![,]>
}

#[derive(Debug)]
#[non_exhaustive]
pub struct KernelAttrs {
    pub attrs: Vec<Attribute>,
    pub link_name: Option<LitStr>
}

impl syn::parse::Parse for KernelAttrs {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = Attribute::parse_outer(input)?;
        let mut link_name = None;

        for i in 0..attrs.len() {
            if attrs[i].path.is_ident("link_name") {
                let attr = attrs.remove(i);
                let token = &attr.tokens;

                let parse : LinkName = parse_quote_spanned! { token.span() => #token };
                link_name = Some(parse.lit);

                break
            }
        }

        Ok(Self { attrs, link_name })
    }
}

#[derive(Parse)]
struct LinkName {
    #[allow(unused)]
    eq_token: Token![=],
    lit: LitStr
}