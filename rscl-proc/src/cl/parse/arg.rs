use std::fmt::Display;

use derive_syn_parse::Parse;
use proc_macro2::Ident;
use syn::{parse::Parse, custom_keyword, Token, Visibility, VisPublic, VisCrate};
use super::r#type::Type;

custom_keyword!(global);
custom_keyword!(local);

#[derive(Parse)]
pub struct FnArg {
    pub qualifier: AddrQualifier,
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Type
}

pub enum AddrQualifier {
    Global (VisPublic),
    Local (VisCrate),
    Const (Token![const]),
    Private
}

impl Parse for AddrQualifier {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![const]) {
            return input.parse::<Token![const]>().map(Self::Const)
        }

        let v = match input.parse::<Visibility>()? {
            Visibility::Public(x) => Self::Global(x),
            Visibility::Crate(x) => Self::Local(x),
            Visibility::Inherited => Self::Private,
            other => return Err(syn::Error::new_spanned(other, "invalid address qualifier"))
        };

        Ok(v)
    }
}

impl Display for AddrQualifier {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Global(_) => f.write_str("__global"),
            Self::Local(_) => f.write_str("__local"),
            Self::Const(_) => f.write_str("__constant"),
            Self::Private => Ok(())
        }
    }
}