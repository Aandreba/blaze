use std::{borrow::Cow, fmt::{Debug, Display}};

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{parse::Parse, bracketed, parse_quote, LitInt, Token};

#[derive(Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Type {
    Void,
    Bool,
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    Float,
    Double,
    Size, // usize

    Array {
        elem: Box<Type>,
        len: LitInt
    },

    Buffer {
        reference: Token![&],
        mutability: Option<Token![mut]>,
        elem: Box<Type>
    }
}

impl Type {
    pub fn to_rust (&self) -> syn::Type {
        match self {
            Self::Void => parse_quote! { () },
            Self::Bool => parse_quote! { bool },
            Self::Char => parse_quote! { i8 },
            Self::UChar => parse_quote! { u8 },
            Self::Short => parse_quote! { i16 },
            Self::UShort => parse_quote! { u16 },
            Self::Int => parse_quote! { i32 },
            Self::UInt => parse_quote! { u32 },
            Self::Long => parse_quote! { i64 },
            Self::ULong => parse_quote! { u64 },
            Self::Float => parse_quote! { f32 },
            Self::Double => parse_quote! { f64 },
            Self::Size => parse_quote! { usize },

            Self::Array { elem, len, .. } => {
                let elem = elem.to_rust();
                parse_quote! { [#elem; #len] }
            }

            Self::Buffer { elem, mutability, .. } => todo!()
        }
    }

    pub fn to_cl (&self) -> [Cow<'static, str>; 3] {
        match self {
            Self::Void => [Cow::Borrowed("void"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Bool => [Cow::Borrowed("bool"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Char => [Cow::Borrowed("char"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::UChar => [Cow::Borrowed("uchar"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Short => [Cow::Borrowed("short"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::UShort => [Cow::Borrowed("ushort"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Int => [Cow::Borrowed("int"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::UInt => [Cow::Borrowed("uint"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Long => [Cow::Borrowed("long"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::ULong => [Cow::Borrowed("ulong"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Float => [Cow::Borrowed("float"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Double => [Cow::Borrowed("double"), Cow::Borrowed(""), Cow::Borrowed("")],
            Self::Size => [Cow::Borrowed("size_t"), Cow::Borrowed(""), Cow::Borrowed("")],

            Self::Array { elem, len, .. } => {
                let [ty, pre, post] = elem.to_cl();
                let post = format!("{post}[{len}]");
                [ty, pre, Cow::Owned(post)]
            },

            Self::Buffer { elem, .. } => {
                let [ty, pre, post] = elem.to_cl();
                let ty = format!("{ty}*");
                [Cow::Owned(ty), pre, post]
            }
        }
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content; bracketed!(content in input);

            let elem = Box::new(content.parse()?);
            content.parse::<Token![;]>()?;
            let len = content.parse()?;

            return Ok(Self::Array { elem, len })
        }

        if input.peek(Token![&]) {
            let reference = input.parse()?;
            let mutability = input.parse()?;

            let content; bracketed!(content in input);
            let elem = Box::new(content.parse()?);

            return Ok(Self::Buffer { reference, mutability, elem })
        }

        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();
        let v = match name.as_str() {
            "()" | "void" => Self::Void,
            "bool" => Self::Bool,
            "i8" => Self::Char,
            "u8" => Self::UChar,
            "i16" => Self::Short,
            "u16" => Self::UShort,
            "i32" => Self::Int,
            "u32" => Self::UInt,
            "i64" => Self::Long,
            "u64" => Self::ULong,
            "f32" => Self::Float,
            "f64" => Self::Double,
            "usize" => Self::Size,
            _ => return Err(syn::Error::new_spanned(ident, "invalid type"))
        };

        Ok(v)
    }
}

impl ToTokens for Type {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.to_rust().to_tokens(tokens)
    }
}

impl Debug for Type {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stream = self.to_rust().to_token_stream();
        Display::fmt(&stream, f)
    }
}