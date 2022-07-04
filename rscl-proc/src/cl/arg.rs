use std::ops::Deref;
use derive_syn_parse::Parse;
use proc_macro2::{Ident};
use quote::{format_ident, quote};
use syn::ext::IdentExt;
use syn::{Token, parse_quote, Generics, GenericParam};
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
    pub fn ty (&self, generics: Option<&mut Generics>) -> syn::Type {
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
            Type::Pointer(ref ty) => {
                let ty: syn::Type = match ty.deref() {
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
                    _ => unimplemented!()
                };

                let (mutability, gen_ty) = match self.constness {
                    None => (Some(syn::token::Mut::default()), quote! { ::rscl::buffer::WriteablePointer<#ty> }),
                    Some(_) => (None, quote! { ::rscl::buffer::ReadablePointer<#ty> })
                };

                let name = format_ident!("{}", self.name.to_string().to_uppercase());
                if let Some(generics) = generics {
                    let generic = parse_quote! { #name: #gen_ty };
                    generics.params.push(GenericParam::Type(generic));
                }

                return parse_quote! { &'a #mutability #name };
            }
        }
    }
}