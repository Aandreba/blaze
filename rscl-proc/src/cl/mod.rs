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

pub fn rscl_c (rscl: Rscl) -> TokenStream {
    let Rscl { vis, struct_token, ident, kernels, .. } = rscl;
    let (str, kernels) = kernels;

    let kernel_names = kernels.iter().map(|x| &x.name).collect::<Vec<_>>();
    let kernel_structs = kernels.iter().map(|x| create_kernel(&vis, &ident, x));
    let kernel_defs = kernels.iter().map(|x| {
        let name = &x.name;
        quote!(#name: ::std::sync::Mutex<::rscl::core::Kernel>)
    });

    quote! {
        #vis #struct_token #ident<C: ::rscl::context::Context = ::rscl::context::Global> {
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
                let (inner, kernels) = ::rscl::core::Program::from_source_in(&ctx, #str, options)?;

                #(let mut #kernel_names = None);*;
                for kernel in kernels.into_iter() {
                    let name = kernel.name()?;
                    match name.as_str() {
                        #(stringify!(#kernel_names) => #kernel_names = unsafe { Some(kernel.clone()) }),*,
                        _ => return Err(::rscl::core::Error::InvalidKernel)
                    }
                }

                #(let #kernel_names = ::std::sync::Mutex::new(#kernel_names.ok_or(::rscl::core::Error::InvalidKernel)?));*;
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

fn create_kernel (vis: &Visibility, parent: &Ident, kernel: &Kernel) -> TokenStream {
    let Kernel { name, args, .. } = kernel;
    let mut generics : Generics = parse_quote! { <'a> };

    let big_name = format_ident!("{}", to_pascal_case(&name.to_string()));
    let define = args.iter().filter_map(|x| define_arg(x, &mut generics)).collect::<Vec<_>>();
    let new = args.iter().map(new_arg);
    let names = args.iter().filter_map(|x| if x.ty.is_pointer() { Some(&x.name) } else { None });
    let set = args.iter().enumerate().map(|(i, x)| set_arg(x, u32::try_from(i).unwrap()));
    let (r#impl, r#type, r#where) = generics.split_for_impl();

    let mut fn_generics : Generics = parse_quote! { #r#impl };
    fn_generics.params.push(parse_quote! { const N: usize });
    let (fn_impl, _, _) = fn_generics.split_for_impl();

    quote! {
        #vis struct #big_name #r#type #r#where {
            inner: ::rscl::event::RawEvent,
            #[doc(hidden)]
            phtm: ::core::marker::PhantomData<&'a ()>,
            #(#define),*
        }

        impl<C: ::rscl::context::Context> #parent<C> {
            pub fn #name #fn_impl (&self, #(#new),*, global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: impl Into<::rscl::event::WaitList>) -> ::rscl::core::Result<#big_name #r#type> #r#where {
                let mut kernel = self.#name.lock().unwrap();
                #(#set);*;
                
                let inner = kernel.enqueue_with_context(&self.ctx, global_work_dims, local_work_dims, wait)?;
                drop(kernel);

                Ok(#big_name {
                    inner,
                    phtm: ::core::marker::PhantomData,
                    #(#names),*
                })
            }
        }

        impl #r#impl ::rscl::event::Event for #big_name #r#type #r#where {
            type Output = ();

            #[inline(always)]
            fn consume (self) -> Self::Output {
                // noop
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
            unsafe { #name.set_argument(&mut kernel, #idx)? }
        }
    }

    quote! { kernel.set_argument(#idx, &#name)? }
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
    pub kernels: (String, Vec<Kernel>)
}

fn parse_kernels (input: syn::parse::ParseStream) -> syn::Result<(String, Vec<Kernel>)> {
    let str = input.fork().parse::<TokenStream>()?.to_string();
    let mut kernels = Vec::with_capacity(1);
    while !input.is_empty() {
        kernels.push(input.parse()?);
    }

    Ok((str, kernels))
}