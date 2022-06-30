pub mod kernel;
pub mod signature;
pub mod arg;
pub mod r#type;
pub mod expr;

use proc_macro2::TokenStream;
use quote::quote;
use self::kernel::Kernel;

pub fn rscl (items: Kernel) -> TokenStream {
    let Kernel { sig } = items;

    quote! {

    }
}