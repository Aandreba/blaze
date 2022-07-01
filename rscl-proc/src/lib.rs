#![feature(is_some_with, extend_one, iter_advance_by, pattern)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

use cl::Kernel;
use error::Error;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse_macro_input, ItemStatic};

mod context;
mod error;
mod utils;
mod cl;

#[proc_macro_attribute]
pub fn global_context (attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let alloc = parse_macro_input!(attrs as Option<Ident>);
    let items = parse_macro_input!(items as ItemStatic);

    let alloc = alloc.is_some_and(|x| x == "alloc");
    context::global_context(items, alloc).into()
}

#[proc_macro]
pub fn error (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(items as Error);
    input.to_token_stream().into()
}

#[proc_macro]
pub fn rscl (items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let str = items.to_string();
    let items = parse_macro_input!(items as Kernel);
    cl::rscl(str, items).into()
}