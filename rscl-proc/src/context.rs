use proc_macro2::{TokenStream};
use quote::quote;
use syn::{ItemStatic};

#[inline(always)]
pub fn global_context (input: ItemStatic) -> TokenStream {
    let ItemStatic { attrs, vis, static_token, mutability, ident, colon_token, ty, eq_token, expr, semi_token } = input;

    quote! {
        #(#attrs)*
        #vis #static_token #mutability #ident #colon_token ::blaze::once_cell::sync::Lazy<#ty> #eq_token ::blaze::once_cell::sync::Lazy::new(|| #expr.unwrap()) #semi_token

        #[doc(hidden)]
        #[no_mangle]
        extern "Rust" fn __blaze__global__as_raw () -> &'static ::blaze::context::RawContext {
            ::blaze::context::Context::as_raw(::blaze::once_cell::sync::Lazy::force(&#ident))
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "Rust" fn __blaze__global__queues () -> &'static [::blaze::core::RawCommandQueue] {
            ::blaze::context::Context::queues(::blaze::once_cell::sync::Lazy::force(&#ident))
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "Rust" fn __blaze__global__next_queue () -> &'static ::blaze::core::RawCommandQueue {
            ::blaze::context::Context::next_queue(::blaze::once_cell::sync::Lazy::force(&#ident))
        }
    }
}