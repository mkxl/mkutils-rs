use proc_macro::TokenStream;
use syn::{ItemFn, LitStr};

pub fn context(args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
    let context = syn::parse_macro_input!(args_token_stream as LitStr);
    let ItemFn { attrs, vis, sig, block } = syn::parse_macro_input!(input_token_stream);
    let output = &sig.output;
    let block_tokens = if sig.asyncness.is_some() {
        quote::quote! {
            ::anyhow::Context::context((async move #block).await, #context)
        }
    } else {
        quote::quote! {
            ::anyhow::Context::context((move || #output #block)(), #context)
        }
    };
    let item_fn_tokens = quote::quote! {
        #(#attrs)*
        #vis #sig {
            #block_tokens
        }
    };

    item_fn_tokens.into()
}
