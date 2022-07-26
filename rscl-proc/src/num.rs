use std::{borrow::Cow};
use proc_macro2::{TokenStream, Ident};
use quote::{quote, ToTokens, format_ident};
use syn::{DeriveInput, Data, Fields, Field, Path, parse_quote};

// Add, Sub, Mul, Div, Rem
pub fn derive_ops (items: DeriveInput) -> TokenStream {
    let mut result = derive_op(&items, parse_quote!(::core::ops::Add), format_ident!("add"));
    derive_op(&items, parse_quote!(::core::ops::Sub), format_ident!("sub")).to_tokens(&mut result);
    derive_op(&items, parse_quote!(::core::ops::Mul), format_ident!("mul")).to_tokens(&mut result);
    derive_op(&items, parse_quote!(::core::ops::Div), format_ident!("div")).to_tokens(&mut result);
    derive_op(&items, parse_quote!(::core::ops::Rem), format_ident!("rem")).to_tokens(&mut result);

    result
}

pub fn derive_ops_assign (items: DeriveInput) -> TokenStream {
    let mut result = derive_op_assign(&items, parse_quote!(::core::ops::AddAssign), format_ident!("add_assign"));
    derive_op_assign(&items, parse_quote!(::core::ops::SubAssign), format_ident!("sub_assign")).to_tokens(&mut result);
    derive_op_assign(&items, parse_quote!(::core::ops::MulAssign), format_ident!("mul_assign")).to_tokens(&mut result);
    derive_op_assign(&items, parse_quote!(::core::ops::DivAssign), format_ident!("div_assign")).to_tokens(&mut result);
    derive_op_assign(&items, parse_quote!(::core::ops::RemAssign), format_ident!("rem_assign")).to_tokens(&mut result);

    result
}

pub fn derive_op (items: &DeriveInput, path: Path, fun: Ident) -> TokenStream {
    let DeriveInput { ident, generics, data, .. } = items;
    let mut generics = Cow::Borrowed(generics);

    if !generics.params.is_empty() {
        for param in generics.to_mut().type_params_mut() {
            let idt = &param.ident;
            param.bounds.push(parse_quote! { #path <#idt, Output = #idt> })
        }
    }

    let (imp, ty, wher) = generics.split_for_impl();
    let inner = impl_derive(ident, data, &path, &fun);

    quote! {
        #[automatically_derived]
        impl #imp #path for #ident #ty #wher {
            type Output = Self;

            #[inline(always)]
            fn #fun (self, rhs: Self) -> Self::Output {
                #inner
            }
        }
    }
}

pub fn derive_op_assign (items: &DeriveInput, path: Path, fun: Ident) -> TokenStream {
    let DeriveInput { ident, generics, data, .. } = items;
    let mut generics = Cow::Borrowed(generics);

    if !generics.params.is_empty() {
        for param in generics.to_mut().type_params_mut() {
            let idt = &param.ident;
            param.bounds.push(parse_quote! { #path <#idt> })
        }
    }

    let (imp, ty, wher) = generics.split_for_impl();
    let inner = impl_derive_assign(ident, data, &path, &fun);

    quote! {
        #[automatically_derived]
        impl #imp #path for #ident #ty #wher {
            #[inline(always)]
            fn #fun (&mut self, rhs: Self) {
                #inner
            }
        }
    }
}

fn impl_derive (ident: &Ident, data: &Data, path: &Path, fun: &Ident) -> TokenStream {
    match data {
        Data::Struct(x) => impl_fields(&Path::from(ident.clone()), &x.fields, path, fun).unwrap_or_else(|| quote! { #ident {} }),
        _ => unimplemented!()
    }
}

fn impl_derive_assign (ident: &Ident, data: &Data, path: &Path, fun: &Ident) -> TokenStream {
    match data {
        Data::Struct(x) => impl_fields_assign(&Path::from(ident.clone()), &x.fields, path, fun),
        _ => unimplemented!()
    }
}

#[inline(always)]
fn impl_fields (path: &Path, fields: &Fields, op_path: &Path, fun: &Ident) -> Option<TokenStream> {
    match &fields {
        Fields::Named(x) => {
            let iter = x.named.iter().map(|x| impl_field(x, None, op_path, fun));
            
            Some(quote! { 
                #path { 
                    #(#iter),*
                }
            })
        },

        Fields::Unnamed(x) => {
            let iter = x.unnamed.iter().enumerate().map(|(i, x)| impl_field(x, Some(i), op_path, fun));
            
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
fn impl_fields_assign (path: &Path, fields: &Fields, op_path: &Path, fun: &Ident) -> TokenStream {
    fields.into_iter()
        .enumerate()
        .map(|(i, x)| impl_field_assign(x, Some(i), op_path, fun))
        .collect()
}

#[inline]
fn impl_field (field: &Field, idx: Option<usize>, path: &Path, fun: &Ident) -> TokenStream {
    let Field { attrs, ident, colon_token, .. } = field; 

    if attrs.contains(&parse_quote! { #[uninit] }) {
        return quote! {
            #(#attrs)*
            #ident #colon_token ::core::mem::MaybeUninit::uninit() 
        }
    }

    let name = match ident {
        Some(x) => x.to_token_stream(),
        None => syn::Index::from(idx.unwrap_or_default()).to_token_stream()
    };

    quote! {
        #(#attrs)*
        #ident #colon_token #path::#fun(self.#name, rhs.#name) 
    }
}

#[inline]
fn impl_field_assign (field: &Field, idx: Option<usize>, path: &Path, fun: &Ident) -> TokenStream {
    let Field { attrs, ident, colon_token, .. } = field; 

    if attrs.contains(&parse_quote! { #[uninit] }) {
        return TokenStream::new()
    }

    let name = match ident {
        Some(x) => x.to_token_stream(),
        None => syn::Index::from(idx.unwrap_or_default()).to_token_stream()
    };

    quote! {
        #(#attrs)*
        #path::#fun(&mut self.#name, rhs.#name); 
    }
}