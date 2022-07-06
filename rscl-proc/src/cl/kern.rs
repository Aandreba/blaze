use derive_syn_parse::Parse;
use proc_macro2::{Ident};
use syn::{punctuated::Punctuated, Token, custom_keyword, LitStr, Visibility};
use super::{Argument};

/*
    kernel void add (const ulong n, __global const float* rhs, __global const float* in, __global float* out) {
        for (ulong id = get_global_id(0); id<n; id += get_global_size(0)) {
            int two = (int)in[id];
            out[id] = in[id] + rhs[id];
        }
    }
*/

custom_keyword!(link_name);

#[derive(Debug, Parse)]
pub struct Kernel {
    #[peek(Token![#])]
    pub attrs: Option<LinkName>,
    pub vis: Visibility,
    pub fn_token: Token![fn],
    pub ident: Ident,
    #[paren]
    pub paren_token: syn::token::Paren,
    #[inside(paren_token)]
    #[call(Punctuated::parse_terminated)]
    pub args: Punctuated<Argument, Token![,]>
}

#[derive(Debug, Parse)]
pub struct LinkName {
    pub pound_token: Token![#],
    #[bracket]
    pub bracket_token: syn::token::Bracket,
    #[inside(bracket_token)]
    pub link_name: link_name,
    #[inside(bracket_token)]
    pub eq_token: Token![=],
    #[inside(bracket_token)]
    pub lit: LitStr
}