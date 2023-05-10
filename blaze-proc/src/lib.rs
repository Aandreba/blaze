#![allow(clippy::all)]

macro_rules! flat_mod {
    ($($i:ident),+) => {
        $(
            mod $i;
            pub use $i::*;
        )+
    }
}

use cl::Link;
use derive_syn_parse::Parse;
use error::Error;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse_macro_input, parse_quote, punctuated::Punctuated, DeriveInput, Generics, ItemStatic,
    ItemType, Meta, Visibility, WhereClause, WherePredicate,
};

use crate::cl::Blaze;

mod cl;
mod context;
mod error;
mod num;
mod utils;

#[proc_macro_derive(NumOps, attributes(uninit))]
pub fn derive_num_ops(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as DeriveInput);
    num::derive_ops(items).into()
}

#[proc_macro_derive(NumOpsAssign, attributes(uninit))]
pub fn derive_num_ops_assign(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as DeriveInput);
    num::derive_ops_assign(items).into()
}

#[proc_macro_attribute]
pub fn global_context(
    _attrs: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as ItemStatic);
    context::global_context(items).into()
}

#[proc_macro]
pub fn error(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(items as Error);
    input.to_token_stream().into()
}

#[proc_macro_attribute]
pub fn newtype(
    attrs: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    fn extra_where(where_generics: Option<&WhereClause>, extra: WherePredicate) -> WhereClause {
        match where_generics {
            Some(x) => {
                let mut x = x.clone();
                x.predicates.push(extra);
                return x;
            }

            None => {
                let mut predicates = Punctuated::new();
                predicates.push(extra);

                WhereClause {
                    where_token: Default::default(),
                    predicates,
                }
            }
        }
    }

    let inner_vis = parse_macro_input!(attrs as Visibility);
    let ItemType {
        attrs,
        vis,
        ident,
        generics,
        ty,
        semi_token,
        ..
    } = parse_macro_input!(items as ItemType);
    let (impl_generics, ty_generics, where_generics) = generics.split_for_impl();

    let consumer_generics = extra_where(
        r#where_generics,
        parse_quote! { #ty: blaze_rs::event::Consumer },
    );
    let debug_generics = extra_where(r#where_generics, parse_quote! { #ty: ::core::fmt::Debug });
    let clone_generics = extra_where(r#where_generics, parse_quote! { #ty: ::core::clone::Clone });
    let copy_generics = extra_where(r#where_generics, parse_quote! { #ty: ::core::marker::Copy });

    quote! {
        #(#attrs)*
        #vis struct #ident #impl_generics (#inner_vis #ty) #semi_token

        impl #impl_generics blaze_rs::event::Consumer for #ident #ty_generics #consumer_generics {
            type Output = <#ty as blaze_rs::event::Consumer>::Output;

            #[inline(always)]
            unsafe fn consume (self) -> blaze_rs::prelude::Result<Self::Output> {
                <#ty as blaze_rs::event::Consumer>::consume(self.0)
            }
        }

        impl #impl_generics ::core::fmt::Debug for #ident #ty_generics #debug_generics {
            #[inline(always)]
            fn fmt (&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Debug::fmt(&self.0, f)
            }
        }

        impl #impl_generics ::core::clone::Clone for #ident #ty_generics #clone_generics {
            #[inline(always)]
            fn clone (&self) -> Self {
                Self(::core::clone::Clone::clone(&self.0))
            }
        }

        impl #impl_generics ::core::marker::Copy for #ident #ty_generics #copy_generics {}
    }
    .into()
}

#[proc_macro]
pub fn join_various_blocking(items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    #[derive(Parse)]
    struct Input(#[call(Punctuated::parse_terminated)] Punctuated<syn::Expr, syn::Token![,]>);

    let item = parse_macro_input!(items as Input).0.into_iter();
    let idx = (0..item.len()).map(syn::Index::from).collect::<Vec<_>>();

    quote! {{
        let v = (#(blaze_rs::event::Event::into_parts(#item)),*);
        let (raw, consumer) = ([#(v.#idx.0),*], (#(v.#idx.1),*));
        blaze_rs::event::RawEvent::join_all_by_ref(&raw).and_then(|_| unsafe {
            Ok((
                #(
                    blaze_rs::event::Consumer::consume(consumer.#idx)?
                ),*
            ))
        })
    }}
    .into()
}

#[proc_macro_attribute]
pub fn blaze(
    attrs: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ident = parse_macro_input!(attrs as BlazeIdent);
    let items = parse_macro_input!(items as Blaze);

    let mut inner = None;
    for attr in &items.attrs {
        if attr.path.is_ident(&format_ident!("link")) {
            let tokens = attr.tokens.clone().into();
            let link = parse_macro_input!(tokens as Link);
            inner = Some(link.meta);
            break;
        }
    }

    if let Some(inner) = inner {
        return cl::blaze_c(ident.vis, ident.ident, ident.generics, items, inner).into();
    }

    panic!("No source code specified");
}

#[proc_macro_attribute]
pub fn docfg(
    attrs: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = parse_macro_input!(attrs as Meta);
    let items = parse_macro_input!(items as TokenStream);

    quote! {
        #[cfg_attr(docsrs, doc(cfg(#attrs)))]
        #[cfg(#attrs)]
        #items
    }
    .into()
}

#[derive(Parse)]
struct BlazeIdent {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
}
