use derive_syn_parse::Parse;
use proc_macro2::Ident;
use quote::{ToTokens, quote};
use syn::{Visibility, Token, parse::Parse, punctuated::Punctuated, Variant, braced, parse_quote};
use crate::utils::AttributeList;

#[derive(Parse)]
pub struct Error {
    attrs: AttributeList,
    vis: Visibility,
    enum_token: Token![enum],
    ident: Ident,
    body: ErrorBody
}

impl ToTokens for Error {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { attrs, vis, enum_token, ident, body } = self;
        let mut attrs = attrs.clone();
        attrs.0.push(parse_quote! { #[repr(i32)] });

        attrs.to_tokens(tokens);
        vis.to_tokens(tokens);
        enum_token.to_tokens(tokens);
        ident.to_tokens(tokens);
        body.to_tokens(tokens);

        let cl_errors = body.inner.iter().filter_map(|x| {
            if let ErrorVariant::OpenCL(x) = x {
                return Some(quote!(opencl_sys::#x))
            }

            None
        }).collect::<Vec<_>>();

        let rust_errors = body.inner.iter().filter_map(|x| {
            if let ErrorVariant::Rust(x) = x {
                let v = &x.discriminant.as_ref().unwrap().1;
                return Some(v.to_token_stream())
            }

            None
        });

        tokens.extend(quote! {
            impl #ident {
                const MIN : i32 = min_value([#(#cl_errors),*]);
                const MAX : i32 = max_value([#(#cl_errors),*]);
            }

            impl Into<i32> for #ident {
                #[inline(always)]
                fn into(self) -> i32 {
                    self as i32
                }
            }
            
            impl From<i32> for #ident {
                #[inline(always)]
                fn from(value: i32) -> Self {
                    match value {
                        Self::MIN..=Self::MAX | #(#rust_errors)|* => unsafe { core::mem::transmute(value) },
                        _ => panic!("invalid error code: {}", value)
                    }
                }
            }
        })
    }
}

struct ErrorBody {
    inner: Punctuated<ErrorVariant, Token![,]>
}

impl Parse for ErrorBody {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content; braced!(content in input);
        let inner = content.parse_terminated(ErrorVariant::parse)?;

        Ok(Self {
            inner
        })
    }
}

impl ToTokens for ErrorBody {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ErrorBody { inner, .. } = self;
        tokens.extend(quote! {{
            #inner
        }})
    }
}

enum ErrorVariant {
    Rust (Variant),
    OpenCL (Ident)
}

impl Parse for ErrorVariant {
    #[inline]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![const]) {
            input.parse::<Token![const]>()?;
            return input.parse().map(Self::OpenCL);
        }

        input.parse().map(Self::Rust)
    }
}

impl ToTokens for ErrorVariant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Rust (var) => var.to_tokens(tokens),
            Self::OpenCL (ident) => {
                let name = ident.to_string();
                let name = Ident::new(&to_pascal_case(&name[3..]), ident.span());
                
                tokens.extend(quote! {
                    #name = opencl_sys::#ident
                })
            }
        }
    }
}

fn to_pascal_case (v: &str) -> String {
    let mut result = String::with_capacity(v.len());
    let mut uppercase = true;

    for c in v.chars() {
        if c == '_' {
            uppercase = true;
            continue
        }

        if uppercase {
            result.extend(c.to_uppercase());
            uppercase = false;
            continue
        }

        result.extend(c.to_lowercase());
    }

    result
}