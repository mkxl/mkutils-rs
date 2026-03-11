use crate::utils::{Cat3, CommaPunctuated, IdentAssignment};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Error as SynError, Expr, ItemFn,
    parse::{Parse, ParseStream},
};

pub struct BuilderCalls {
    builder_calls: Vec<TokenStream2>,
}

impl BuilderCalls {
    fn builder_call(Cat3(ident, _comma, expr): IdentAssignment<Expr>) -> TokenStream2 {
        quote::quote! { .#ident(#expr) }
    }
}

impl Parse for BuilderCalls {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        let assignments = CommaPunctuated::<IdentAssignment<Expr>>::parse_terminated(parse_stream)?;
        let builder_calls = assignments.into_iter().map(Self::builder_call).collect();
        let builder_calls = Self { builder_calls };

        Ok(builder_calls)
    }
}

pub fn main(attr_args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
    let BuilderCalls { builder_calls } = syn::parse_macro_input!(attr_args_token_stream as BuilderCalls);
    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = syn::parse_macro_input!(input_token_stream);

    sig.asyncness = None;

    // NOTE: [https://docs.rs/tokio/latest/tokio/attr.main.html#using-the-multi-threaded-runtime]
    let main_fn_token_stream = quote::quote! {
        #(#attrs)*
        #vis #sig {
            ::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                #(#builder_calls)*
                .build()
                .unwrap()
                .block_on(async #block)
        }
    };

    main_fn_token_stream.into()
}
