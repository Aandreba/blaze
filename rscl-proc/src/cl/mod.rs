use proc_macro2::TokenStream;
use quote::quote;

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

pub fn rscl (str: String, rscl: Kernel) -> TokenStream {
    let Kernel { name, out, args } = rscl;
    let define = args.iter().map(define_arg);

    quote! {
        pub struct #name<'a> {
            #[doc(hidden)]
            phtm: ::core::marker::PhantomData<&'a ()>,
            #(#define),*
        }
    }
}

fn define_arg (arg: &Argument) -> TokenStream {
    let Argument { name, .. } = arg;
    let ty = arg.def_ty();

    quote! {
        #name: #ty
    }
}