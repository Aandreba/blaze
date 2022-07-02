use derive_syn_parse::Parse;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Visibility, Token, Generics, parse_quote};
use crate::utils::to_pascal_case;

macro_rules! peek_and_parse {
    ($i:ident in $input:expr) => {{
        let v = $input.peek($i);
        if v {
            let _ = $input.parse::<$i>()?;
        }

        v
    }}
}

flat_mod!(ty, kern, access, arg);

pub fn rscl (str: String, rscl: Rscl) -> TokenStream {
    let Rscl { vis, struct_token, ident, kernels, .. } = rscl;
    let kernels = kernels.into_iter().map(|x| create_kernel(&vis, x));

    quote! {
        #vis #struct_token #ident<'a> {
            #[doc(hidden)]
            phtm: ::core::marker::PhantomData<&'a ()>,
        }

        #(#kernels)*
    }
}

fn create_kernel (vis: &Visibility, kernel: Kernel) -> TokenStream {
    let Kernel { name, out, args } = kernel;
    let mut generics : Generics = parse_quote! { <'a> };

    let name = format_ident!("{}", to_pascal_case(&name.to_string()));
    let define = args.iter().map(|x| define_arg(x, &mut generics)).collect::<Vec<_>>();
    let new = args.iter().map(new_arg);
    let names = args.iter().map(|x| &x.name);

    let (r#impl, r#type, r#where) = generics.split_for_impl();

    quote! {
        #vis struct #name #r#type #r#where {
            #[doc(hidden)]
            phtm: ::core::marker::PhantomData<&'a ()>,
            #(#define),*
        }

        impl #r#impl #name #r#type #r#where {
            #vis fn new (#(#new),*) -> Self {
                Self {
                    phtm: ::core::marker::PhantomData,
                    #(#names),*
                }
            }
        }
    }
}

fn define_arg (arg: &Argument, generics: &mut Generics) -> TokenStream {
    let Argument { name, .. } = arg;
    let ty = arg.ty(Some(generics));

    quote! {
        #name: #ty
    }
}

fn new_arg (arg: &Argument) -> TokenStream {
    let Argument { name, .. } = arg;
    let ty = arg.ty(None);

    quote! {
        #name: #ty
    }
}

#[derive(Parse)]
pub struct Rscl {
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    #[brace]
    pub brace_token: syn::token::Brace,
    #[inside(brace_token)]
    #[call(parse_kernels)]
    pub kernels: Vec<Kernel>
}

fn parse_kernels (input: syn::parse::ParseStream) -> syn::Result<Vec<Kernel>> {
    let mut kernels = Vec::with_capacity(1);
    while !input.is_empty() {
        kernels.push(input.parse()?);
    }

    Ok(kernels)
}