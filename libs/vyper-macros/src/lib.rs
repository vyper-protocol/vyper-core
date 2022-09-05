use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn log_wrap_ix(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let stmts = &block.stmts;
    let name = &sig.ident;
    let gen = quote! {
        #(#attrs)* #vis #sig {
            msg!("{} begin", stringify!(#name));

            #(#stmts)*
        }
    };

    gen.into()
}
