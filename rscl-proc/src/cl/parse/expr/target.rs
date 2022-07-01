use std::fmt::Display;

use derive_syn_parse::Parse;
use proc_macro2::Ident;
use syn::bracketed;
use super::{Expr, utils::Inferr};

#[derive(Parse)]
pub struct Variable {
    pub ident: Ident,
    #[call(parse_idx)]
    pub idx: Option<Box<Expr>>,
}

impl Display for Variable {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { ident, idx } = self;
        if let Some(idx) = idx {
            return write!(f, "{ident}[{idx}]");
        }

        write!(f, "{ident}")
    }
}

impl Inferr for Variable {
    #[inline(always)]
    fn inferrence (&self) -> super::utils::Inferrence {
        super::utils::Inferrence::None
    }
}

fn parse_idx (input: syn::parse::ParseStream) -> syn::Result<Option<Box<Expr>>> {
    if input.peek(syn::token::Bracket) {
        let content; bracketed!(content in input);
        let expr = content.parse()?;
        return Ok(Some(Box::new(expr)))
    }

    Ok(None)
}