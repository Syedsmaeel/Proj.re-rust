use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr};

#[proc_macro]
pub fn sushi_draw(input: TokenStream) -> TokenStream {
    // This macro "diffuses" your UI code into direct canvas drawing instructions
    let input = parse_macro_input!(input as Expr);
    
    // We transform the declaration into a series of tiny-skia draw calls
    quote! {
        {
            #input
        }
    }.into()
}
