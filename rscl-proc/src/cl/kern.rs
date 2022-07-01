use proc_macro2::{Ident, TokenStream};
use syn::{parse::Parse, custom_keyword, parenthesized, punctuated::Punctuated, Token, braced};
use super::{Argument, Type};

/*
    kernel void add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            int two = (int)in[id];
            out[id] = in[id] + rhs[id];
        }
    }
*/

custom_keyword!(kernel);
custom_keyword!(__kernel);

#[derive(Debug)]
pub struct Kernel {
    pub name: Ident,
    pub out: Type,
    pub args: Punctuated<Argument, Token![,]>
}

impl Parse for Kernel {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(__kernel) {
            input.parse::<__kernel>()?;
        } else {
            input.parse::<kernel>()?;
        }

        let out = input.parse()?;
        let name = input.call(syn::ext::IdentExt::parse_any)?;

        let content; parenthesized!(content in input);
        let args = content.parse_terminated(Argument::parse)?;

        let content; braced!(content in input);
        while !content.is_empty() {
            let _ = content.parse::<TokenStream>()?;
        }

        Ok(Self { name, out, args })
    }
}