use std::fmt::Display;
use derive_syn_parse::Parse;
use proc_macro2::Ident;
use crate::cl::arg::AddrQualifier;
use super::*;

#[derive(Parse)]
pub struct Local {
    pub vis: AddrQualifier,
    pub let_token: Token![let],
    pub ident: Ident,
    #[call(Local::parse_ty)]
    pub ty: Option<(Token![:], Type)>,
    pub eq_token: Token![=],
    pub expr: Expr,
    pub semi_token: Token![;]
}

impl Local {
    pub fn parse_ty (input: syn::parse::ParseStream) -> syn::Result<Option<(Token![:], Type)>> {
        if input.peek(Token![:]) {
            let colon_token = input.parse()?;
            let ty = input.parse()?;
            return Ok(Some((colon_token, ty)))
        }

        Ok(None)
    }
}

impl Inferr for Local {
    #[inline(always)]
    fn inferrence (&self) -> Inferrence {
        if let Some((_, ref ty)) = self.ty {
            return Inferrence::Strong(ty.clone())
        }

        self.expr.inferrence()
    }
}

impl Display for Local {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { vis, ident, expr, .. } = self;

        let ty = self.inferrence();
        let ty = ty.ty().unwrap();
        
        let [ty, pre, post] = ty.to_cl();
        write!(f, "{vis} {ty} {pre}{ident}{post} = {expr};")
    }
}