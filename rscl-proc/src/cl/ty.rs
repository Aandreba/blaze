use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use syn::{parse::Parse, LitInt, TypePath, token::{Mut, Star}, bracketed, parse_quote_spanned, spanned::Spanned, Token, Generics, GenericParam, WherePredicate};

/*
    kernel void add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            int two = (int)in[id];
            out[id] = in[id] + rhs[id];
        }
    }
*/

#[derive(Debug)]
pub enum Type {
    Array (Box<Type>, LitInt),
    Path (TypePath),
    Pointer (bool, Box<Type>)
}

impl Type {
    #[inline(always)]
    pub const fn is_pointer (&self) -> ::std::primitive::bool {
        match self {
            Self::Pointer { .. } => true,
            _ => false
        }
    }

    pub fn rustify (&self, name: &Ident) -> (Option<(GenericParam, WherePredicate)>, syn::Type) {
        match self {
            Type::Array(ty, len) => {
                let (gen, ty) = ty.rustify(name);
                let v = parse_quote_spanned! { ty.span() => [#ty; #len] };
                (gen, v)
            },

            Type::Path(ty) => (None, syn::Type::Path(ty.clone())),

            Type::Pointer(mutability, ty) => {
                let ty = ty.rustify_ptr();
                let wher = parse_quote_spanned! { ty.span() => <#name as ::core::ops::Deref>::Target: ::rscl::buffer::KernelPointer<#ty> };
                let param = match mutability {
                    true => parse_quote_spanned! { ty.span() => #name: ::core::ops::DerefMut },
                    false => parse_quote_spanned! { ty.span() => #name: ::core::ops::Deref }
                };

                (Some((param, wher)), parse_quote_spanned! { name.span() => #name })
            }
        }
    }

    pub fn rustify_ptr (&self) -> syn::Type {
        match self {
            Self::Array(ty, _) => ty.rustify_ptr(),
            Self::Path(x) => syn::Type::Path(x.clone()),
            Self::Pointer(_, ty) => todo!(),
        }
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if peek_and_parse!(Star in input) {
            let mutability = peek_and_parse!(Mut in input);
            if !mutability { let _ = input.parse::<Token![const]>()?; }

            let ty = Box::new(input.parse()?);
            return Ok(Self::Pointer(mutability, ty))
        }

        if input.peek(syn::token::Bracket) {
            let content; bracketed!(content in input);
            let ty = Box::new(content.parse()?);
            let _ = input.parse::<Token![;]>()?;
            let len = content.parse()?;
            return Ok(Self::Array(ty, len))
        }

        input.parse().map(Self::Path)
    }
}