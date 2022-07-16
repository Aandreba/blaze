#![feature(is_some_with, extend_one, iter_advance_by, pattern)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

use cl::{Rscl, Link};
use error::Error;
use proc_macro2::{TokenStream, Ident};
use quote::{ToTokens, quote, format_ident};
use syn::{parse_macro_input, ItemStatic, Meta};

mod context;
mod error;
mod utils;
mod cl;

#[proc_macro_attribute]
pub fn global_context (_attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as ItemStatic);
    context::global_context(items).into()
}

#[proc_macro]
pub fn error (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(items as Error);
    input.to_token_stream().into()
}

#[proc_macro_attribute]
pub fn rscl (attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ident = parse_macro_input!(attrs as Ident);
    let items = parse_macro_input!(items as Rscl);

    let mut inner = None;
    for attr in &items.attrs {
        if attr.path.is_ident(&format_ident!("link")) {
            let tokens = attr.tokens.clone().into();
            let link = parse_macro_input!(tokens as Link);
            inner = Some(link.meta);
            break
        }
    }

    if let Some(inner) = inner {
        return cl::rscl_c(ident, items, inner).into()
    }

    panic!("No source code specified");
}

#[proc_macro_attribute]
pub fn docfg (attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let attrs = parse_macro_input!(attrs as Meta);
    let items = parse_macro_input!(items as TokenStream);

    quote! {
        #[cfg_attr(docsrs, doc(cfg(#attrs)))]
        #[cfg(#attrs)]
        #items
    }.into()
}