use derive_syn_parse::Parse;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident, ToTokens};
use syn::{Visibility, Token, Generics, parse_quote, Abi, punctuated::Punctuated, Attribute, Expr};
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

pub fn rscl_c (ident: Ident, rscl: Rscl, content: Expr) -> TokenStream {
    let Rscl { vis, kernels, .. } = rscl;

    let kernel_names = kernels.iter().map(|x| &x.ident).collect::<Vec<_>>();
    let kernel_extern_names = kernels.iter().map(|x| {
        if let Some(name) = &x.attrs { 
            return name.lit.to_token_stream() 
        }

        let ident = &x.ident; 
        quote! { stringify!(#ident) }
    }).collect::<Vec<_>>();

    let kernel_structs = kernels.iter().map(|x| create_kernel(&vis, &ident, x));
    let kernel_defs = kernels.iter().map(|x| {
        let name = &x.ident;
        quote!(#name: ::std::sync::Mutex<::rscl::core::Kernel>)
    });

    quote! {
        #vis struct #ident<C: ::rscl::context::Context = ::rscl::context::Global> {
            inner: ::rscl::core::Program,
            ctx: C,
            #(#kernel_defs),*
        }

        impl #ident<::rscl::context::Global> {
            #[inline(always)]
            #vis fn new<'a> (options: impl Into<Option<&'a str>>) -> ::rscl::core::Result<Self> {
                Self::new_in(::rscl::context::Global, options)
            }
        }

        impl<C: ::rscl::context::Context> #ident<C> {
            #vis fn new_in<'a> (ctx: C, options: impl Into<Option<&'a str>>) -> ::rscl::core::Result<Self> {
                let (inner, kernels) = ::rscl::core::Program::from_source_in(&ctx, #content, options)?;

                #(let mut #kernel_names = None);*;
                for kernel in kernels.into_iter() {
                    let name = kernel.name()?;
                    match name.as_str() {
                        #(#kernel_extern_names => #kernel_names = unsafe { Some(kernel.clone()) }),*,
                        __other => return Err(::rscl::core::Error::new(::rscl::core::ErrorType::InvalidKernel, format!("unknown kernel '{}'", __other)))
                    }
                }

                #(
                    let #kernel_names = match #kernel_names {
                        Some(__x) => ::std::sync::Mutex::new(__x),
                        None => return Err(::rscl::core::Error::new(::rscl::core::ErrorType::InvalidKernel, concat!("kernel '", stringify!(#kernel_names), "' not found")))
                    };
                )*

                Ok(Self {
                    inner,
                    ctx,
                    #(#kernel_names),*
                })
            }
        }

        impl<C: ::rscl::context::Context> ::std::ops::Deref for #ident<C> {
            type Target = ::rscl::core::Program;

            #[inline(always)]
            fn deref (&self) -> &Self::Target {
                &self.inner
            }
        }

        #(#kernel_structs)*
    }
}

fn create_kernel (parent_vis: &Visibility, parent: &Ident, kernel: &Kernel) -> TokenStream {
    let Kernel { vis, ident, args, .. } = kernel;
    let mut generics = Generics::default();
    let vis = match vis {
        Visibility::Inherited => parent_vis,
        other => other
    };

    let big_name = format_ident!("{}", to_pascal_case(&ident.to_string()));
    let define = args.iter().filter_map(|x| define_arg(x, &mut generics)).collect::<Vec<_>>();
    let new = args.iter().map(new_arg);
    let names = args.iter().filter_map(|x| if x.ty.is_pointer() { Some(&x.name) } else { None }).collect::<Vec<_>>();
    let set = args.iter().enumerate().map(|(i, x)| set_arg(x, u32::try_from(i).unwrap()));
    let (r#impl, r#type, r#where) = generics.split_for_impl();
    let type_list = generics.type_params().map(|x| &x.ident);

    let mut fn_generics : Generics = parse_quote! { #r#impl };
    fn_generics.params.push(parse_quote! { const N: usize });
    let (fn_impl, _, _) = fn_generics.split_for_impl();

    quote! {
        #vis struct #big_name #r#type {
            inner: ::rscl::event::RawEvent,
            #(#define),*
        }

        impl<C: ::rscl::context::Context> #parent<C> {
            pub unsafe fn #ident #fn_impl (&self, #(#new),*, global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: impl Into<::rscl::event::WaitList>) -> ::rscl::core::Result<#big_name #r#type> #r#where {
                let mut kernel = self.#ident.lock().unwrap();
                #(#set);*;

                let inner = kernel.enqueue_with_context(&self.ctx, global_work_dims, local_work_dims, wait)?;
                drop(kernel);

                Ok(#big_name {
                    inner,
                    #(#names),*
                })
            }
        }

        impl #r#impl ::rscl::event::Event for #big_name #r#type #r#where {
            type Output = (#(#type_list),*);

            #[inline(always)]
            fn as_raw (&self) -> &::rscl::event::RawEvent {
                &self.inner
            }

            #[inline(always)]
            fn consume (self, err: Option<::rscl::prelude::Error>) -> ::rscl::prelude::Result<Self::Output> {
                if let Some(err) = err { return Err(err) }; 
                Ok((#(self.#names),*))
            }
        }

        impl #r#impl std::convert::AsRef<::rscl::event::RawEvent> for #big_name #r#type #r#where {
            #[inline(always)]
            fn as_ref (&self) -> &::rscl::event::RawEvent {
                &self.inner
            }
        }
    }
}

fn define_arg (arg: &Argument, generics: &mut Generics) -> Option<TokenStream> {
    if arg.ty.is_pointer() {
        let Argument { name, .. } = arg;
        let ty = arg.ty(Some(generics));

        return Some(quote! {
            #name: #ty
        })
    }

    None
}

fn new_arg (arg: &Argument) -> TokenStream {
    let Argument { name, .. } = arg;
    let ty = arg.ty(None);

    return quote! {
        #name: #ty
    }
}

fn set_arg (arg: &Argument, idx: u32) -> TokenStream {
    let Argument { name, .. } = arg;

    if arg.ty.is_pointer() {
        return quote! {
            ::rscl::buffer::KernelPointer::set_arg(::core::ops::Deref::deref(&#name), &mut kernel, #idx)?
        }
    }

    quote! { kernel.set_argument(#idx, &#name)? }
}

#[derive(Parse)]
pub struct Rscl {
    #[call(Attribute::parse_outer)]
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub abi: Abi,
    #[brace]
    pub brace_token: syn::token::Brace,
    #[inside(brace_token)]
    #[call(Punctuated::parse_terminated)]
    pub kernels: Punctuated<Kernel, Token![;]>
}

#[derive(Parse)]
pub struct Link {
    #[paren]
    pub paren_token: syn::token::Paren,
    #[inside(paren_token)]
    pub meta: Expr
}