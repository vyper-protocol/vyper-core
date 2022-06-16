use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};
use quote::quote;

#[proc_macro_attribute]
pub fn log_wrap_ix(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
    let ItemFn { attrs, vis, sig, block } = input;
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

// #[proc_macro_attribute]
// pub fn redeem_logic_interface(_attr: TokenStream, stream: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(stream as ItemFn);

//     let ItemFn { attrs, vis, sig, block } = input;
//     let stmts = &block.stmts;
//     let gen = quote! {
//         #(#attrs)* #vis #sig {
//             #(#stmts)*
            
//             // msg!("{} end", stringify!(#sig));
//         }

//         #[derive(AnchorSerialize, AnchorDeserialize, Debug)]
//         pub struct RedeemLogicExecuteInput {
//             pub old_quantity: [u64; 2],
//             pub old_reserve_fair_value_bps: u32,
//             pub new_reserve_fair_value_bps: u32
//         }

//         #[derive(AnchorSerialize, AnchorDeserialize, Debug)]
//         pub struct RedeemLogicExecuteResult {
//             pub new_quantity: [u64;2],
//             pub fee_quantity: u64
//         }
//     };

//     gen.into()
// }