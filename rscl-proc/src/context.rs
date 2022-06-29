use proc_macro2::{TokenStream};
use quote::quote;
use syn::{ItemStatic};

#[inline(always)]
pub fn global_context (input: ItemStatic, alloc: bool) -> TokenStream {
    let ItemStatic { attrs, vis, static_token, mutability, ident, colon_token, ty, eq_token, expr, semi_token } = input;

    let alloc = alloc.then(|| quote! {
        #[global_allocator]
        static ALLOC : ::rscl::svm::Svm = ::rscl::svm::Svm::new();
    });

    quote! {
        #(#attrs)*
        #vis #static_token #mutability #ident #colon_token ::rscl::once_cell::sync::Lazy<#ty> #eq_token ::rscl::once_cell::sync::Lazy::new(|| #expr) #semi_token

        #[doc(hidden)]
        #[no_mangle]
        extern "Rust" fn __rscl__global__context () -> &'static ::rscl::context::RawContext {
            ::rscl::context::Context::context(::rscl::once_cell::sync::Lazy::force(&#ident))
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "Rust" fn __rscl__global__queue_count () -> usize {
            ::rscl::context::Context::queue_count(::rscl::once_cell::sync::Lazy::force(&#ident))
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "Rust" fn __rscl__global__next_queue () -> &'static ::rscl::core::CommandQueue {
            ::rscl::context::Context::next_queue(::rscl::once_cell::sync::Lazy::force(&#ident))
        }

        #alloc
    }
}