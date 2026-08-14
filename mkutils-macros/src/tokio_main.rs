use crate::utils::{Cat3, CommaPunctuated, IdentAssignment};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Error as SynError, Expr, ItemFn,
    parse::{Parse, ParseStream},
};

pub struct TokioMain {
    method_applications: Vec<TokenStream2>,
}

impl Parse for TokioMain {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        let assignments = CommaPunctuated::<IdentAssignment<Expr>>::parse_terminated(parse_stream)?;
        let method_applications = assignments.into_iter().map(Self::method_application).collect();
        let method_applications = Self { method_applications };

        Ok(method_applications)
    }
}

impl TokioMain {
    fn method_application(Cat3(ident, _equals, expr): IdentAssignment<Expr>) -> TokenStream2 {
        quote::quote! { .#ident(#expr) }
    }

    fn derive_impl(&self, item_fn: ItemFn) -> TokenStream2 {
        let Self { method_applications } = self;
        let ItemFn {
            attrs,
            vis,
            mut sig,
            block,
        } = item_fn;

        sig.asyncness = None;

        // NOTE: [https://docs.rs/tokio/latest/tokio/attr.main.html#using-the-multi-threaded-runtime]
        quote::quote! {
            #(#attrs)*
            #vis #sig {
                ::tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    #(#method_applications)*
                    .build()
                    .unwrap()
                    .block_on(async #block)
            }
        }
    }

    pub fn derive(attr_args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
        let tokio_main = syn::parse_macro_input!(attr_args_token_stream as Self);
        let item_fn = syn::parse_macro_input!(input_token_stream);

        tokio_main.derive_impl(item_fn).into()
    }
}
