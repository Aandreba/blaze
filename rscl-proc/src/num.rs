use std::{borrow::Cow};
use proc_macro2::{TokenStream, Ident};
use quote::{quote, ToTokens, format_ident};
use syn::{DeriveInput, Data, Fields, Field, Variant, Path};

// Add, Sub, Mul, Div, Rem
pub fn derive_ops (items: DeriveInput) -> TokenStream {
    let mut result = derive_add(&items);
    result
}

pub fn derive_add (items: &DeriveInput) -> TokenStream {
    let DeriveInput { attrs, vis, ident, generics, data } = items;
    let (imp, ty, wher) = generics.split_for_impl();

    quote! {
        impl #imp ::core::ops::Add for #ident #ty #wher {
            type Output = Self;

            #[inline(always)]
            fn add (self, rhs: &Self) -> Self::Output {
                todo!()
            }
        }
    }
}

fn impl_derive (ident: &Ident, data: &Data, token: impl ToTokens) -> TokenStream {
    match data {
        Data::Struct(x) => impl_fields(&Path::from(ident.clone()), &x.fields, token).unwrap_or_else(|| quote! { #ident {} }),
        _ => unimplemented!()
    }
}

#[inline(always)]
fn impl_fields (path: &Path, fields: &Fields, token: impl ToTokens) -> Option<TokenStream> {
    match &fields {
        Fields::Named(x) => {
            let iter = x.named.iter().map(|x| impl_field(x, None, &token));
            
            Some(quote! { 
                #path { 
                    #(#iter),*
                }
            })
        },

        Fields::Unnamed(x) => {
            let iter = x.unnamed.iter().enumerate().map(|(i, x)| impl_field(x, Some(i), &token));
            
            Some(quote! { 
                #path (
                    #(#iter),*
                )
            })
        },

        Fields::Unit => None,
    }
}

#[inline(always)]
fn impl_variants<'a, I: IntoIterator<Item = &'a Variant>> (path: &Path, iter: I, token: impl ToTokens) -> TokenStream {
    iter.into_iter().map(|x| impl_variant(path, x, &token)).collect()
}

#[inline]
fn impl_field (field: &Field, idx: Option<usize>, token: impl ToTokens) -> TokenStream {
    let Field { attrs, ident, colon_token, .. } = field;
    let name = match ident {
        Some(x) => Cow::Borrowed(x),
        None => Cow::Owned(format_ident!("{}", idx.unwrap_or_default()))
    }; 

    quote! {
        #(#attrs)*
        #ident #colon_token self.#name #token rhs.#name 
    }
}

fn impl_variant (path: &Path, variant: &Variant, token: impl ToTokens) -> TokenStream {
    todo!()
}