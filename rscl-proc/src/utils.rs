use quote::{ToTokens, TokenStreamExt};
use syn::AttrStyle;
use syn::{Attribute, parse::Parse};

#[derive(Clone)]
#[repr(transparent)]
pub struct AttributeList (pub Vec<Attribute>);

impl AttributeList {
    fn outer(&self) -> impl Iterator<Item = &Attribute> {
        fn is_outer(attr: &&Attribute) -> bool {
            match attr.style {
                AttrStyle::Outer => true,
                AttrStyle::Inner(_) => false,
            }
        }

        self.0.iter().filter(is_outer)
    }
}

impl Parse for AttributeList {
    #[inline(always)]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.call(Attribute::parse_outer).map(Self)
    }
}

impl ToTokens for AttributeList {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(self.outer());
    }
}