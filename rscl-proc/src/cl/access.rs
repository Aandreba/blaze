use syn::{custom_keyword, parse::Parse, token::Const};

custom_keyword!(global);
custom_keyword!(local);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Global,
    Local,
    Const,
    Private
}

impl Parse for Access {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if peek_and_parse!(global in input) {
            return Ok(Self::Global)
        }

        if peek_and_parse!(local in input) {
            return Ok(Self::Local)
        }

        if peek_and_parse!(Const in input) {
            return Ok(Self::Const)
        }

        return Ok(Self::Private)
    }
}