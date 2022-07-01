use syn::Token;
use crate::cl::r#type::Type;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inferrence {
    None,
    Weak (Type),
    Strong (Type)
}

impl Inferrence {
    pub const fn ty (&self) -> Option<&Type> {
        match self {
            Self::Weak(x) | Self::Strong(x) => Some(x),
            _ => None
        }
    }

    #[inline(always)]
    pub const fn is_none (&self) -> bool {
        match self {
            Self::None => true,
            _ => false
        }
    }
}

pub trait Inferr {
    fn inferrence (&self) -> Inferrence;
}

impl<T: Inferr> Inferr for &T {
    #[inline(always)]
    fn inferrence (&self) -> Inferrence {
        T::inferrence(self)
    }
}

impl Inferr for Inferrence {
    #[inline(always)]
    fn inferrence (&self) -> Inferrence {
        self.clone()
    }
}

impl Inferr for Type {
    #[inline(always)]
    fn inferrence (&self) -> Inferrence {
        Inferrence::Strong(self.clone())
    }
}

pub fn peek_bin_op (input: syn::parse::ParseStream) -> bool {
    return 
        input.peek(Token![&&]) |
        input.peek(Token![||]) |
        input.peek(Token![<<]) |
        input.peek(Token![>>]) |
        input.peek(Token![==]) |
        input.peek(Token![<=]) |
        input.peek(Token![!=]) |
        input.peek(Token![>=]) |
        input.peek(Token![+]) |
        input.peek(Token![-]) |
        input.peek(Token![*]) |
        input.peek(Token![/]) |
        input.peek(Token![^]) |
        input.peek(Token![&]) |
        input.peek(Token![|]) |
        input.peek(Token![<]) |
        input.peek(Token![>]) |
        input.peek(Token![+=]) |
        input.peek(Token![-=]) |
        input.peek(Token![*=]) |
        input.peek(Token![/=]) |
        input.peek(Token![%=]) |
        input.peek(Token![^=]) |
        input.peek(Token![&=]) |
        input.peek(Token![|=]) |
        input.peek(Token![<<=]) |
        input.peek(Token![>>=])
}