use derive_syn_parse::Parse;
use proc_macro2::{Ident};
use quote::{format_ident};
use syn::{Generics, Token};
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
    pub fn ty (&self, generics: Option<&mut Generics>) -> syn::Type {
        let name = format_ident!("{}", self.name.to_string().to_uppercase());
        let (generify, ty) = self.ty.rustify(self.mutability.is_some(), &name);

        if let Some((imp, wher)) = generify {
            if let Some(generics) = generics {
                generics.params.push(imp);
                generics.make_where_clause().predicates.push(wher)
            }
        }

        ty
    }
}