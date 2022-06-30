use proc_macro2::Ident;
use syn::{parse::Parse, bracketed};

#[derive(Clone)]
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
    Buffer {
        bracket_token: syn::token::Bracket,
        elem: Box<Type>
    }
}

impl Type {
    pub fn to_rust (&self) -> syn::Type {
        todo!()
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content; 
            let bracket_token = bracketed!(content in input);
            
            let elem = Box::new(content.parse()?);
            return Ok(Self::Buffer { bracket_token, elem })
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