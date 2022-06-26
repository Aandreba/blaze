use proc_macro2::{TokenStream};
use quote::quote;
use syn::{ItemStatic};

#[inline(always)]
pub fn global_context (input: ItemStatic) -> TokenStream {
    let ItemStatic { attrs, vis, static_token, mutability, ident, colon_token, ty, eq_token, expr, semi_token } = input;

    quote! {
        #(#attrs)*
        #vis #static_token #mutability #ident #colon_token ::rscl::once_cell::sync::Lazy<#ty> #eq_token ::rscl::once_cell::sync::Lazy::new(|| #expr) #semi_token

        #[doc(hidden)]
        #[no_mangle]
        pub extern "Rust" fn __rscl__global__context_id () -> *mut ::std::ffi::c_void {
            ::rscl::context::Context::context_id(::rscl::once_cell::sync::Lazy::force(&#ident))
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "Rust" fn __rscl__global__queue_count () -> usize {
            ::rscl::context::Context::queue_count(::rscl::once_cell::sync::Lazy::force(&#ident))
        }

        #[doc(hidden)]
        #[no_mangle]
        pub extern "Rust" fn __rscl__global__next_queue () -> *mut ::std::ffi::c_void {
            ::rscl::context::Context::next_queue(::rscl::once_cell::sync::Lazy::force(&#ident))
        }
    }
}