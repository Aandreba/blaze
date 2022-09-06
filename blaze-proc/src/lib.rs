#![feature(is_some_with, extend_one, iter_advance_by, pattern)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

use cl::{Link};
use derive_syn_parse::Parse;
use error::Error;
use proc_macro2::{TokenStream, Ident};
use quote::{ToTokens, quote, format_ident};
use syn::{parse_macro_input, ItemStatic, Meta, DeriveInput, Generics};

use crate::cl::Blaze;

mod context;
mod error;
mod utils;
mod cl;
mod num;

#[proc_macro_derive(NumOps, attributes(uninit))]
pub fn derive_num_ops (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as DeriveInput);
    num::derive_ops(items).into()
}

#[proc_macro_derive(NumOpsAssign, attributes(uninit))]
pub fn derive_num_ops_assign (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as DeriveInput);
    num::derive_ops_assign(items).into()
}

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

/*
#[proc_macro_attribute]
pub fn blaze (attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ident = parse_macro_input!(attrs as BlazeIdent);
    let items = parse_macro_input!(items as Blaze);

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
        return cl::blaze_c(ident.ident, ident.generics, items, inner).into()
    }

    panic!("No source code specified");
}
*/

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

#[derive(Parse)]
struct BlazeIdent {
    ident: Ident,
    generics: Generics
}