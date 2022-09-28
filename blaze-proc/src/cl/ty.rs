use proc_macro2::{Ident};
use syn::{parse::Parse, LitInt, TypePath, token::{Mut, Star}, bracketed, parse_quote_spanned, spanned::Spanned, Token, GenericParam, custom_keyword, parse_quote};

custom_keyword!(image2d);

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Array (Box<Type>, LitInt),
    Path (TypePath),
    Pointer (bool, Box<Type>),
    Image2d
}

impl Type {
    #[inline(always)]
    pub const fn is_pointer (&self) -> ::std::primitive::bool {
        match self {
            Self::Pointer { .. } => true,
            _ => false
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn is_define (&self) -> ::std::primitive::bool {
        match self {
            Self::Pointer { .. } | Self::Image2d => true,
            _ => false
        }
    }

    pub fn rustify (&self, mutability: bool, name: &Ident) -> (bool, Option<GenericParam>, syn::Type) {
        match self {
            Type::Array(ty, len) => {
                let (_, gen, ty) = ty.rustify(mutability, name);
                let v = parse_quote_spanned! { ty.span() => [#ty; #len] };
                (false, gen, v)
            },

            Type::Path(ty) => (false, None, syn::Type::Path(ty.clone())),

            Type::Pointer(mutability, ty) => {
                let ty = ty.rustify_ptr();
                let param = parse_quote! { #name: ::blaze_rs::buffer::KernelPointer<#ty> };
                (*mutability, Some(param), parse_quote_spanned! { name.span() => #name })
            },

            Type::Image2d => {
                let param = parse_quote! { #name: ::blaze_rs::image::DynImage2D };
                (false, Some(param), parse_quote_spanned! { name.span() => #name })
            },
        }
    }

    pub fn rustify_ptr (&self) -> syn::Type {
        match self {
            Self::Array(ty, _) => ty.rustify_ptr(),
            Self::Path(x) => syn::Type::Path(x.clone()),
            Self::Image2d => todo!(),
            #[allow(unused)]
            Self::Pointer(_, ty) => todo!(),
        }
    }
}

impl Parse for Type {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if peek_and_parse!(Star in input) {
            let mutability = peek_and_parse!(Mut in input);
            if !mutability { let _ = input.parse::<Token![const]>()?; }

            let ty = Box::new(input.parse()?);
            return Ok(Self::Pointer(mutability, ty))
        }

        if input.peek(syn::token::Bracket) {
            let content; bracketed!(content in input);
            let ty = Box::new(content.parse()?);
            let _ = input.parse::<Token![;]>()?;
            let len = content.parse()?;
            return Ok(Self::Array(ty, len))
        }

        if peek_and_parse!(image2d in input) {
            return Ok(Self::Image2d)
        }

        input.parse().map(Self::Path)
    }
}