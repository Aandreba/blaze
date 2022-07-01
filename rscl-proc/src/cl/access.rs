use syn::{custom_keyword, parse::Parse};

custom_keyword!(__global);
custom_keyword!(global);
custom_keyword!(__local);
custom_keyword!(local);
custom_keyword!(__constant);
custom_keyword!(constant);

#[derive(Debug)]
pub enum Access {
    Global,
    Local,
    Const,
    Private   
}

impl Parse for Access {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if peek_and_parse!(__global in input) || peek_and_parse!(global in input) {
            return Ok(Self::Global)
        }

        if peek_and_parse!(__local in input) || peek_and_parse!(local in input) {
            return Ok(Self::Local)
        }

        if peek_and_parse!(__constant in input) || peek_and_parse!(constant in input) {
            return Ok(Self::Const)
        }

        return Ok(Self::Private)
    }
}