mod parse;
pub use parse::*;

pub mod kernel;
pub mod compile;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Generics, Lifetime, parse_str, GenericParam, LifetimeDef};
use self::{kernel::Kernel, signature::Signature, arg::FnArg, r#type::Type, compile::compile, expr::Block};

pub fn rscl (items: Kernel) -> TokenStream {
    let Kernel { sig, block } = items;
    create_structure(&sig, block)
}

fn create_structure (sig: &Signature, block: Block) -> TokenStream {
    let Signature { vis, kernel_token, fn_token, ident, paren_token, inputs } = sig;
    let mut lt = LifetimeCollector::new();
    let define = inputs.iter().map(|x| define_field(x, &mut lt)).collect::<Vec<_>>();
    let comp = compile(block);

    quote! {
        #vis struct #ident #lt {
            #(#define),*
        }

        impl #lt #ident #lt {
            const SOURCE : &'static str = #comp;
        }
    }
}

fn define_field (input: &FnArg, lt: &mut LifetimeCollector) -> TokenStream {
    let FnArg { ident, colon_token, ty, .. } = input;
    let ty = match ty {
        Type::Buffer { mutability, .. } => {
            let life = lt.push();
            quote! { &#life #mutability ::rscl::buffer::RawBuffer }
        },

        ty => ty.to_token_stream()
    };

    quote! {
        #ident #colon_token #ty
    }
}

struct LifetimeCollector {
    inner: Generics,
    current: u8
}

impl LifetimeCollector {
    #[inline(always)]
    pub fn new () -> Self {
        Self { inner: Generics::default(), current: b'a' }
    }

    pub fn push (&mut self) -> Lifetime {
        let current = self.current as char;
        self.current = self.current + 1;

        let lt : Lifetime = parse_str(&format!("'{current}")).unwrap();
        let def = LifetimeDef::new(lt.clone());
        self.inner.params.push(GenericParam::Lifetime(def));
        lt
    }
}

impl ToTokens for LifetimeCollector {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.inner.to_tokens(tokens)
    }
}