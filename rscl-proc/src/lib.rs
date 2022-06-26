use syn::{parse_macro_input, ItemStatic};
mod context;

#[proc_macro_attribute]
pub fn global_context (_attrs: proc_macro::TokenStream, items: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let items = parse_macro_input!(items as ItemStatic);
    context::global_context(items).into()
}