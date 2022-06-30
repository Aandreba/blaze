use std::{fmt::Display};

use derive_syn_parse::Parse;
use proc_macro2::Ident;
use syn::{Token, BinOp};
use super::{Expr, utils::{Inferr}};

#[derive(Parse)]
pub struct ExprAssign {
    pub left: Ident,
    pub eq_token: Token![=],
    pub right: Box<Expr>
}

#[derive(Parse)]
pub struct ExprAssignOp {
    pub left: Ident,
    pub op: BinOp,
    pub right: Box<Expr>
}

impl Display for ExprAssign {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { left, right, .. } = self;
        write!(f, "{left} = {right}")
    }
}

impl Display for ExprAssignOp {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { left, op, right } = self;
        let op = match op {
            BinOp::Add(_) => "+",
            BinOp::AddEq(_) => "+=",
            BinOp::And(_) => "&&",
            BinOp::BitAnd(_) => "&",
            BinOp::BitAndEq(_) => "&=",
            BinOp::BitOr(_) => "|",
            BinOp::BitOrEq(_) => "|=",
            BinOp::BitXor(_) => "^",
            BinOp::BitXorEq(_) => "^=",
            BinOp::Div(_) => "/",
            BinOp::DivEq(_) => "/=",
            BinOp::Eq(_) => "=",
            BinOp::Ge(_) => ">=",
            BinOp::Gt(_) => ">",
            BinOp::Le(_) => "<=",
            BinOp::Lt(_) => "<",
            BinOp::Mul(_) => "*",
            BinOp::MulEq(_) => "*=",
            BinOp::Ne(_) => "!=",
            BinOp::Or(_) => "||",
            BinOp::Rem(_) => "%",
            BinOp::RemEq(_) => "%=",
            BinOp::Shl(_) => "<<",
            BinOp::ShlEq(_) => "<<=",
            BinOp::Shr(_) => ">>",
            BinOp::ShrEq(_) => ">>=",
            BinOp::Sub(_) => "-",
            BinOp::SubEq(_) => "-="
        };

        write!(f, "{left} {op} {right}")
    }
}

impl Inferr for ExprAssign {
    #[inline(always)]
    fn inferrence (&self) -> super::utils::Inferrence {
        self.right.inferrence()
    }
}

impl Inferr for ExprAssignOp {
    #[inline(always)]
    fn inferrence (&self) -> super::utils::Inferrence {
        self.right.inferrence()
    }
}