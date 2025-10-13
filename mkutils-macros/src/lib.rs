use proc_macro::TokenStream;
use syn::{ItemFn, LitStr};

#[proc_macro_attribute]
pub fn context(args_tokens: TokenStream, input_tokens: TokenStream) -> TokenStream {
    let context = syn::parse_macro_input!(args_tokens as LitStr);
    let ItemFn { attrs, vis, sig, block } = syn::parse_macro_input!(input_tokens);
    let block_tokens = if sig.asyncness.is_some() {
        quote::quote! {
            ::anyhow::Context::context((async move #block).await, #context)
        }
    } else {
        quote::quote! {
            ::anyhow::Context::context((move || #block)(), #context)
        }
    };
    let item_fn_tokens = quote::quote! {
        #(#attrs)*
        #vis #sig #block_tokens
    };

    item_fn_tokens.into()
}
