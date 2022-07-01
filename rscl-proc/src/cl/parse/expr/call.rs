use std::fmt::{Display};
use derive_syn_parse::Parse;
use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Token};
use super::{Expr, utils::{Inferrence, Inferr}};

#[derive(Parse)]
pub struct ExprCall {
    pub func: Ident,
    #[paren]
    pub paren_token: syn::token::Paren,
    #[inside(paren_token)]
    #[call(Punctuated::parse_terminated)]
    pub args: Punctuated<Expr, Token![,]>
}

impl Display for ExprCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { func, args, .. } = self;
        if args.len() == 0 {
            return write!(f, "{func}()")
        }

        write!(f, "{func}(")?;
        
        for i in 0..(args.len() - 1) {
            write!(f, "{},", &args[i])?;
        }

        write!(f, "{})", args.last().unwrap())
    }
}

impl Inferr for ExprCall {
    #[inline(always)]
    fn inferrence (&self) -> Inferrence {
        Inferrence::None
    }
}