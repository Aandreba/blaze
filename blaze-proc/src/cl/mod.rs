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

pub fn blaze_c (ident: Ident, generics: Generics, blaze: Blaze, content: Expr) -> TokenStream {
    let Blaze { vis, kernels, .. } = blaze;

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
        quote!(#name: ::std::sync::Mutex<::blaze_rs::core::RawKernel>)
    });

    quote! {
        #vis struct #ident<C: ::blaze_rs::context::Context = ::blaze_rs::context::Global> {
            __blaze_inner__: ::blaze_rs::core::RawProgram,
            __blaze_ctx__: C,
            #(#kernel_defs),*
        }

        impl #ident<::blaze_rs::context::Global> {
            #[inline(always)]
            #vis fn new<'a> (options: impl Into<Option<&'a str>>) -> ::blaze_rs::core::Result<Self> {
                Self::new_in(::blaze_rs::context::Global, options)
            }
        }

        impl<C: ::blaze_rs::context::Context> #ident<C> {
            #vis fn new_in<'a> (ctx: C, options: impl Into<Option<&'a str>>) -> ::blaze_rs::core::Result<Self> {
                let __blaze_ctx__ = ctx;
                let (__blaze_inner__, __blaze_kernels__) = ::blaze_rs::core::RawProgram::from_source_in(&__blaze_ctx__, #content, options)?;

                #(let mut #kernel_names = None);*;
                for __blaze_kernel__ in __blaze_kernels__.into_iter() {
                    match __blaze_kernel__.name()?.as_str() {
                        #(#kernel_extern_names => #kernel_names = unsafe { Some(__blaze_kernel__.clone()) }),*,
                        __other => return Err(::blaze_rs::core::Error::new(::blaze_rs::core::ErrorType::InvalidKernel, format!("unknown kernel '{}'", __other)))
                    }
                }

                #(
                    let #kernel_names = match #kernel_names {
                        Some(__x) => ::std::sync::Mutex::new(__x),
                        None => return Err(::blaze_rs::core::Error::new(::blaze_rs::core::ErrorType::InvalidKernel, concat!("kernel '", stringify!(#kernel_names), "' not found")))
                    };
                )*

                Ok(Self {
                    __blaze_inner__,
                    __blaze_ctx__,
                    #(#kernel_names),*
                })
            }
        }

        impl<C: ::blaze_rs::context::Context> ::std::ops::Deref for #ident<C> {
            type Target = ::blaze_rs::core::RawProgram;

            #[inline(always)]
            fn deref (&self) -> &Self::Target {
                &self.__blaze_inner__
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
    let names = args.iter().filter_map(|x| if x.ty.is_define() { Some(&x.name) } else { None }).collect::<Vec<_>>();
    let pointer_names = args.iter().filter_map(|x| if x.ty.is_pointer() { Some(&x.name) } else { None });
    let set = args.iter().enumerate().map(|(i, x)| set_arg(x, u32::try_from(i).unwrap()));
    let (r#impl, r#type, r#where) = generics.split_for_impl();
    let type_list = generics.type_params().map(|x| &x.ident);

    let mut fn_generics : Generics = parse_quote! { #r#impl };
    fn_generics.params.push(parse_quote! { const N: usize });
    let (fn_impl, _, _) = fn_generics.split_for_impl();

    quote! {
        #vis struct #big_name #r#type {
            __blaze_inner__: ::blaze_rs::event::RawEvent,
            #(#define),*
        }

        impl<C: ::blaze_rs::context::Context> #parent<C> {
            #vis unsafe fn #ident #fn_impl (&self, #(#new,)* global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: impl Into<::blaze_rs::event::WaitList>) -> ::blaze_rs::core::Result<#big_name #r#type> #r#where {
                let mut wait = wait.into();
                let mut __blaze_kernel__ = match self.#ident.lock() {
                    Ok(x) => x,
                    Err(e) => e.into_inner()
                };

                #(#set);*;

                let __blaze_inner__ = __blaze_kernel__.enqueue_with_context(&self.__blaze_ctx__, global_work_dims, local_work_dims, wait)?;
                drop(__blaze_kernel__);

                #(
                    ::blaze_rs::buffer::KernelPointer::complete(::core::ops::Deref::deref(&#pointer_names), &__blaze_inner__)?;
                )*

                Ok(#big_name {
                    __blaze_inner__,
                    #(#names),*
                })
            }
        }

        impl #r#impl ::blaze_rs::event::Event for #big_name #r#type #r#where {
            type Output = (#(#type_list),*);

            #[inline(always)]
            fn as_raw (&self) -> &::blaze_rs::event::RawEvent {
                &self.__blaze_inner__
            }

            #[inline(always)]
            fn consume (self, err: Option<::blaze_rs::prelude::Error>) -> ::blaze_rs::prelude::Result<Self::Output> {
                if let Some(err) = err { return Err(err) }; 
                Ok((#(self.#names),*))
            }
        }
    }
}

fn define_arg (arg: &Argument, generics: &mut Generics) -> Option<TokenStream> {
    if arg.ty.is_define() {
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

    match arg.ty {
        Type::Pointer(_, _) => quote! {
            ::blaze_rs::buffer::KernelPointer::set_arg(::core::ops::Deref::deref(&#name), &mut __blaze_kernel__, &mut wait, #idx)?
        },

        Type::Image2d => quote! { __blaze_kernel__.set_argument(#idx, ::blaze_rs::image::DynImage2D::id_ref(::core::ops::Deref::deref(&#name)))? },
        _ => quote! { __blaze_kernel__.set_argument(#idx, &#name)? }
    }
}

#[derive(Parse)]
pub struct Blaze {
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
    pub eq_token: Token![=],
    pub meta: Expr
}