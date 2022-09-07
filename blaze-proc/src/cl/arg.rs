use derive_syn_parse::Parse;
use proc_macro2::{Ident};
use quote::{format_ident};
use syn::{Generics, Token, parse_quote, parse_quote_spanned, spanned::Spanned};
use super::{Type};

/*
    kernel void add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            int two = (int)in[id];
            out[id] = in[id] + rhs[id];
        }
    }
*/

#[derive(Debug, Parse)]
pub struct Argument {
    pub mutability : Option<Token![mut]>,
    pub name: Ident,
    pub semi_token: Token![:],
    pub ty: Type,
}

impl Argument {
    pub fn ty (&self, generics: &mut Generics) -> syn::Type {
        let name = format_ident!("{}", self.name.to_string().to_uppercase());
        let (mutability, generify, ty) = self.ty.rustify(self.mutability.is_some(), &name);

        if let Some(imp) = generify {
            generics.params.push(imp);
            
            return match mutability {
                true => parse_quote_spanned! { ty.span() => &'__env__ mut #ty },
                false => parse_quote_spanned! { ty.span() => &'__env__ #ty }
            }
        }

        ty
    }
}