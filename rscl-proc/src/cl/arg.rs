use std::hint::unreachable_unchecked;

use derive_syn_parse::Parse;
use proc_macro2::Ident;
use quote::quote;
use syn::ext::IdentExt;
use syn::{Token, parse_quote, Generics};
use super::{Access, Type};

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
    pub access: Access,
    pub constness: Option<Token![const]>,
    pub ty: Type,
    #[call(syn::Ident::parse_any)]
    pub name: Ident
}

impl Argument {
    pub fn def_ty (&self) -> syn::Type {
        if let Type::Pointer(ref x) = self.ty {
            let mutability = match self.constness {
                None => Some(syn::token::Mut::default()),
                Some(_) => None
            };

            return parse_quote! { &'a #mutability ::rscl::buffer::RawBuffer };
        }

        match self.ty {
            Type::Void => parse_quote! { () },
            Type::Bool => parse_quote! { bool },
            Type::Char => parse_quote! { i8 },
            Type::UChar => parse_quote! { u8 },
            Type::Short => parse_quote! { i16 },
            Type::UShort => parse_quote! { u16 },
            Type::Int => parse_quote! { i32 },
            Type::UInt => parse_quote! { u32 },
            Type::Long => parse_quote! { i64 },
            Type::ULong => parse_quote! { u64 },
            Type::Float => parse_quote! { f32 },
            Type::Double => parse_quote! { f64 },
            Type::Pointer(_) => unsafe { unreachable_unchecked() }
        }
    }
}