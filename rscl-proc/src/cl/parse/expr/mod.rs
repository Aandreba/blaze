flat_mod!(target, local, array, assign, call);
pub mod utils;

use self::{local::Local, utils::{Inferrence, Inferr, peek_bin_op}};
use std::{fmt::{Display, Write, Pointer}};
use syn::{Token, parse::Parse, braced, LitInt, LitFloat, LitBool};
use super::{r#type::Type};

pub struct Block {
    pub brace_token: syn::token::Brace,
    pub stmts: Vec<Stmt>
}

impl Parse for Block {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let brace_token = braced!(content in input);

        let mut stmts = Vec::with_capacity(1);
        while !content.is_empty() {
            let stmt : Stmt = content.parse()?;
            stmts.push(stmt);
        }

        Ok(Self { brace_token, stmts })
    }
}

impl Display for Block {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;
        
        for stmt in self.stmts.iter() {
            stmt.fmt(f)?;
        }

        f.write_char('}')
    }
}

#[non_exhaustive]
pub enum Stmt {
    Local (Local),
    Expr (Expr, Token![;])
}

impl Parse for Stmt {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![let]) {
            return input.parse().map(Self::Local)
        }

        let expr = input.parse()?;
        let semi = input.parse()?;
        Ok(Self::Expr (expr, semi))
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       match self {
           Self::Local(x) => x.fmt(f),
           Self::Expr (x, _) => write!(f, "{x};")
       }
    }
}

#[non_exhaustive]
pub enum Expr {
    Array (ExprArray),
    Assign (ExprOp),
    Break (Token![break]),
    Call (ExprCall),
    Variable (Variable),
    Lit (Lit)
}

impl Inferr for Expr {
    fn inferrence (&self) -> Inferrence {
        match self {
            Self::Array(x) => x.inferrence(),
            Self::Assign(x) => x.inferrence(),
            Self::Lit(x) => x.inferrence(),
            Self::Call(x) => x.inferrence(),
            Self::Variable (x) => x.inferrence(),
            Self::Break(_) => Inferrence::None
        }
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![break]) {
            return input.parse().map(Self::Break)
        }

        if input.peek(syn::token::Bracket) {
            return input.parse().map(Self::Array)
        }

        if input.peek(syn::Ident) {
            if input.peek2(syn::token::Paren) {
                return input.parse().map(Self::Call)
            }

            panic!("{input:?}");
            if input.peek2(Token![=]) || peek_bin_op(input) {
                return input.parse().map(Self::Assign)
            }

            return input.parse().map(Self::Variable)
        }

        input.parse().map(Self::Lit)
    }
}

impl Display for Expr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(x) => x.fmt(f),
            Self::Assign(x) => x.fmt(f),
            Self::Lit(x) => x.fmt(f),
            Self::Call(x) => x.fmt(f),
            Self::Variable(x) => x.fmt(f),
            Self::Break(_) => f.write_str("break")
        }
    }
}

pub enum Lit {
    Int (LitInt),
    Float (LitFloat),
    Bool (LitBool)
}

impl Inferr for Lit {
    fn inferrence (&self) -> utils::Inferrence {
        match self {
            Self::Int(x) => match x.suffix() {
                "" => Inferrence::Weak(Type::Int),
                "i8" => Inferrence::Strong(Type::Char),
                "u8" => Inferrence::Strong(Type::UChar),
                "i16" => Inferrence::Strong(Type::Short),
                "u16" => Inferrence::Strong(Type::UShort),
                "i32" => Inferrence::Strong(Type::Int),
                "u32" => Inferrence::Strong(Type::UInt),
                "i64" => Inferrence::Strong(Type::Long),
                "u64" => Inferrence::Strong(Type::ULong),
                "usize" => Inferrence::Strong(Type::Size),
                _ => panic!("Invalid type"),
            },

            Self::Float(x) => match x.suffix() {
                "" => Inferrence::Weak(Type::Double),
                "f32" => Inferrence::Strong(Type::Float),
                "f64" => Inferrence::Strong(Type::Double),
                _ => panic!("Invalid type"),
            },

            Self::Bool(_) => Inferrence::Strong(Type::Bool)
        }
    }
}

impl Parse for Lit {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<syn::Lit>()?;
        let v = match lit {
            syn::Lit::Int(x) => Self::Int(x),
            syn::Lit::Float(x) => Self::Float(x),
            syn::Lit::Bool(x) => Self::Bool(x),
            other => return Err(syn::Error::new_spanned(other, "invalid literal"))
        };

        Ok(v)
    }
}

impl Display for Lit {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(x) => f.write_str(x.base10_digits()),
            Self::Float(x) => f.write_str(x.base10_digits()),
            Self::Bool(x) => x.fmt(f)
        }
    }
}