use std::fmt::{Display, Write};
use derive_syn_parse::Parse;
use proc_macro2::Literal;
use syn::{punctuated::Punctuated, Token, LitInt};
use crate::cl::r#type::Type;

use super::{Expr, utils::{Inferr, Inferrence}};

#[derive(Parse)]
pub struct ExprArray {
    #[bracket]
    pub bracket_token: syn::token::Bracket,
    #[inside(bracket_token)]
    #[call(Punctuated::parse_terminated)]
    pub elems: Punctuated<Expr, Token![,]>
}

impl Display for ExprArray {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.elems.len() == 0 {
            return f.write_str("{}")
        }

        f.write_char('{')?;
        
        for i in 0..(self.elems.len() - 1) {
            write!(f, "{},", &self.elems[i])?;
        }

        write!(f, "{}}}", self.elems.last().unwrap())
    }
}

impl Inferr for ExprArray {
    fn inferrence (&self) -> super::utils::Inferrence {
        let mut inferr = None;
        for elem in self.elems.iter() {
            match elem.inferrence() {
                x @ Inferrence::Strong(_) => return x,
                Inferrence::Weak(x) => match inferr {
                    Some(y) if x != y => panic!("Incompatible inferrences: {x:?} v. {y:?}"),
                    _ => inferr = Some(x)
                },

                _ => {}
            }
        }

        match inferr {
            Some(x) => {
                let len = Literal::usize_unsuffixed(self.elems.len());
                Inferrence::Weak(Type::Array { elem: Box::new(x), len: LitInt::from(len) })
            },
            None => Inferrence::None
        }
    }
}