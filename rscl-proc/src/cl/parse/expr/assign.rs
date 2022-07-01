use std::{fmt::Display};
use derive_syn_parse::Parse;
use elor::prelude::*;
use syn::{Token, BinOp};
use super::{Expr, utils::{Inferr}, Variable};

#[derive(Parse)]
pub struct ExprOp {
    pub left: Variable,
    #[call(parse_op)]
    pub op: Either<Token![=], BinOp>,
    pub right: Box<Expr>
}

impl Display for ExprOp {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { left, op, right } = self;
        
        let op = match op {
            Left(_) => "=",
            Right(BinOp::Add(_)) => "+",
            Right(BinOp::AddEq(_)) => "+=",
            Right(BinOp::And(_)) => "&&",
            Right(BinOp::BitAnd(_)) => "&",
            Right(BinOp::BitAndEq(_)) => "&=",
            Right(BinOp::BitOr(_)) => "|",
            Right(BinOp::BitOrEq(_)) => "|=",
            Right(BinOp::BitXor(_)) => "^",
            Right(BinOp::BitXorEq(_)) => "^=",
            Right(BinOp::Div(_)) => "/",
            Right(BinOp::DivEq(_)) => "/=",
            Right(BinOp::Eq(_)) => "=",
            Right(BinOp::Ge(_)) => ">=",
            Right(BinOp::Gt(_)) => ">",
            Right(BinOp::Le(_)) => "<=",
            Right(BinOp::Lt(_)) => "<",
            Right(BinOp::Mul(_)) => "*",
            Right(BinOp::MulEq(_)) => "*=",
            Right(BinOp::Ne(_)) => "!=",
            Right(BinOp::Or(_)) => "||",
            Right(BinOp::Rem(_)) => "%",
            Right(BinOp::RemEq(_)) => "%=",
            Right(BinOp::Shl(_)) => "<<",
            Right(BinOp::ShlEq(_)) => "<<=",
            Right(BinOp::Shr(_)) => ">>",
            Right(BinOp::ShrEq(_)) => ">>=",
            Right(BinOp::Sub(_)) => "-",
            Right(BinOp::SubEq(_)) => "-="
        };

        write!(f, "{left} {op} {right}")
    }
}

impl Inferr for ExprOp {
    #[inline(always)]
    fn inferrence (&self) -> super::utils::Inferrence {
        self.right.inferrence()
    }
}

#[inline(always)]
fn parse_op (input: syn::parse::ParseStream) -> syn::Result<Either<Token![=], BinOp>> {
    if input.peek(Token![=]) {
        return input.parse().map(Left)
    }

    input.parse().map(Right)
}