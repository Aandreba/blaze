use std::fmt::Display;
use proc_macro2::Ident;
use crate::cl::arg::AddrQualifier;
use super::*;

pub struct Local {
    pub vis: AddrQualifier,
    pub let_token: Token![let],
    pub ident: Ident,
    pub ty: Type,
    pub eq_token: Token![=],
    pub expr: Expr,
    pub semi_token: Token![;]
}

impl Parse for Local {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let let_token = input.parse()?;
        let ident = input.parse()?;
        
        let mut ty : Option<Type> = None;
        if input.peek(Token![:]) {
            let _ = input.parse::<Token![:]>()?;
            ty = input.parse().map(Some)?;
        }

        let eq_token = input.parse()?;
        let expr = input.parse::<Expr>()?;

        if ty.is_none() {
            match expr.inferrence() {
                Inferrence::Weak(x) | Inferrence::Strong(x) => ty = Some(x),
                Inferrence::None => return Err(syn::Error::new_spanned(ident, "variable type could not be inferred"))
            }
        }

        let semi_token = input.parse()?;

        Ok(Self {
            vis,
            let_token,
            ident,
            ty: ty.unwrap(),
            eq_token,
            expr,
            semi_token
        })
    }
}

impl Inferr for Local {
    #[inline(always)]
    fn inferrence (&self) -> Inferrence {
        self.ty.inferrence()
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