use derive_syn_parse::Parse;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident, ToTokens};
use syn::{Visibility, Token, Generics, parse_quote, Abi, punctuated::Punctuated, Attribute, Expr, GenericParam};
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
    let phantom_generics = match generics.params.is_empty() {
        true => None,
        false => {
            let ty = generics.type_params().map(|p| p.ident.to_token_stream());
            let lt = generics.lifetimes().map(|p| quote! { &#p () });

            let iter = ty.chain(lt);
            Some(quote! { #[doc(hidden)] __blaze_phtm__: ::core::marker::PhantomData::<(#(#iter),*)>,})
        }
    };

    let phantom_fill = phantom_generics.as_ref().map(|_| quote! { __blaze_phtm__: ::core::marker::PhantomData, });

    let mut program_generics = generics.clone();
    program_generics.params.push(parse_quote!(C: ::blaze_rs::context::Context = ::blaze_rs::context::Global));
    let (prog_imp, prog_ty, prog_wher) = program_generics.split_for_impl();
    let (glob_imp, glob_ty, glob_wher) = generics.split_for_impl();

    let kernel_names = kernels.iter().map(|x| &x.ident).collect::<Vec<_>>();
    let kernel_extern_names = kernels.iter().map(|x| {
        if let Some(name) = &x.attrs { 
            return name.lit.to_token_stream() 
        }

        let ident = &x.ident; 
        quote! { stringify!(#ident) }
    }).collect::<Vec<_>>();

    let kernel_structs = kernels.iter()
        .map(|x| create_kernel(&vis, &ident, &generics, &program_generics, &phantom_fill, x));

    let kernel_defs = kernels.iter().map(|x| {
        let name = &x.ident;
        quote!(#name: ::std::sync::Mutex<::blaze_rs::core::RawKernel>)
    });

    quote! {
        #vis struct #ident #program_generics {
            #[doc(hidden)]
            __blaze_inner__: ::blaze_rs::core::RawProgram,
            #[doc(hidden)]
            __blaze_ctx__: C,
            #phantom_generics
            #(#kernel_defs),*
        }

        impl #glob_imp #ident #glob_ty #glob_wher {
            #[inline(always)]
            #vis fn new<'a> (options: impl Into<Option<&'a str>>) -> ::blaze_rs::core::Result<Self> {
                Self::new_in(::blaze_rs::context::Global, options)
            }
        }

        impl #prog_imp #ident #prog_ty #prog_wher {
            #vis fn new_in<'a> (ctx: C, options: impl Into<Option<&'a str>>) -> ::blaze_rs::core::Result<Self> {
                let __blaze_ctx__ = ctx;
                let (__blaze_inner__, __blaze_kernels__) = ::blaze_rs::core::RawProgram::from_source_in(&__blaze_ctx__, #content, options)?;

                #(let mut #kernel_names = None);*;
                for __blaze_kernel__ in __blaze_kernels__.into_iter() {
                    match __blaze_kernel__.name()?.as_str() {
                        #(#kernel_extern_names => #kernel_names = unsafe { Some(__blaze_kernel__.clone()) }),*,
                        _ => {}
                        //__other => return Err(::blaze_rs::core::Error::new(::blaze_rs::core::ErrorType::InvalidKernel, format!("unknown kernel '{}'", __other)))
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
                    #phantom_fill
                    #(#kernel_names),*
                })
            }
        }

        impl #prog_imp ::std::ops::Deref for #ident #prog_ty #prog_wher {
            type Target = ::blaze_rs::core::RawProgram;

            #[inline(always)]
            fn deref (&self) -> &Self::Target {
                &self.__blaze_inner__
            }
        }

        #(#kernel_structs)*
    }
}

fn create_kernel (parent_vis: &Visibility, parent: &Ident, impl_generics: &Generics, parent_generics: &Generics, phantom_fill: &Option<TokenStream>, kernel: &Kernel) -> TokenStream {
    let Kernel { vis, ident, args, .. } = kernel;
    let mut generics = parse_quote! { <'__scope__, '__env__, const N: usize> };
    let (parent_imp, parent_ty, parent_wher) = parent_generics.split_for_impl();

    let vis = match vis {
        Visibility::Inherited => parent_vis,
        other => other
    };

    let big_name = format_ident!("{}", to_pascal_case(&ident.to_string()));
    let new = args.iter().map(|x| new_arg(x, &mut generics)).collect::<Vec<_>>();
    let names = args.iter().filter_map(|x| if x.ty.is_define() { Some(&x.name) } else { None }).collect::<Vec<_>>();
    let pointer_names = args.iter().filter_map(|x| if x.ty.is_pointer() { Some(&x.name) } else { None });
    let set = args.iter().enumerate().map(|(i, x)| set_arg(x, u32::try_from(i).unwrap()));
    generics.params.extend(impl_generics.params.iter().cloned());
    let (r#impl, _, r#where) = generics.split_for_impl();

    quote! {
        impl #parent_imp #parent #parent_ty #parent_wher {
            #vis unsafe fn #ident #r#impl (&self, scope: &'__scope__ ::blaze_rs::context::Scope<'__scope__, '__env__, C>, #(#new,)* global_work_dims: [usize; N], local_work_dims: impl Into<Option<[usize; N]>>, wait: &[::blaze_rs::event::RawEvent]) -> ::blaze_rs::prelude::Result<::blaze_rs::event::NoopEvent<'__scope__>> #r#where {
                let mut wait = wait.to_vec();
                let mut __blaze_kernel__ = match self.#ident.lock() {
                    Ok(x) => x,
                    Err(e) => e.into_inner()
                };

                #(#set);*;

                let __blaze_inner__ = __blaze_kernel__.enqueue_with_context(&self.__blaze_ctx__, global_work_dims, local_work_dims, &wait)?;
                drop(__blaze_kernel__);

                #(
                    ::blaze_rs::buffer::KernelPointer::complete(#pointer_names, &__blaze_inner__)?;
                )*

                return Ok(::blaze_rs::event::NoopEvent::new_noop(__blaze_inner__))
            }
        }
    }
}

fn new_arg (arg: &Argument, generics: &mut Generics) -> TokenStream {
    let Argument { name, .. } = arg;
    let ty = arg.ty(generics);

    return quote! {
        #name: #ty
    }
}

fn set_arg (arg: &Argument, idx: u32) -> TokenStream {
    let Argument { name, .. } = arg;

    match arg.ty {
        Type::Pointer(_, _) => quote! {
            ::blaze_rs::buffer::KernelPointer::set_arg(#name, &mut __blaze_kernel__, &mut wait, #idx)?
        },

        Type::Image2d => quote! { __blaze_kernel__.set_argument(#idx, ::blaze_rs::image::DynImage2D::id_ref(#name))? },
        _ => quote! { __blaze_kernel__.set_argument(#idx, #name)? }
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