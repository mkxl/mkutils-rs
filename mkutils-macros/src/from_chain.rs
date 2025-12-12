use crate::utils::CommaPunctuated;
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{DeriveInput, Error as SynError, Ident, Type, spanned::Spanned};

pub struct FromChain;

impl FromChain {
    const ATTRIBUTE_NAME: &str = "from";

    fn empty_chain(span: Span) -> SynError {
        let message = std::format!(
            "empty `{attribute_name}` attribute found",
            attribute_name = Self::ATTRIBUTE_NAME
        );

        SynError::new(span, message)
    }

    fn from_impl_block_token_stream(
        input_ident: &Ident,
        type_chain: &CommaPunctuated<Type>,
    ) -> Result<TokenStream2, SynError> {
        let mut type_chain_pairs = type_chain.pairs();
        let Some(head_type) = type_chain_pairs.next() else { Err(Self::empty_chain(type_chain.span()))? };
        let head_ident = quote::quote! { value };
        let mut from_chain = quote::quote! { #head_ident };

        while let Some(type_pair) = type_chain_pairs.next_back() {
            let type_ = type_pair.into_value();

            from_chain = quote::quote! { #type_::from(#from_chain) };
        }

        let from_impl_block_token_stream = quote::quote! {
            impl ::std::convert::From<#head_type> for #input_ident {
                fn from(#head_ident: #head_type) -> Self {
                    Self::from(#from_chain)
                }
            }
        };

        Ok(from_impl_block_token_stream)
    }

    #[allow(clippy::similar_names)]
    pub fn derive_impl(derive_input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let mut from_impl_blocks_token_stream = TokenStream2::new();

        for attribute in &derive_input.attrs {
            if attribute.path().is_ident(Self::ATTRIBUTE_NAME) {
                let type_chain = attribute.parse_args_with(CommaPunctuated::<Type>::parse_terminated)?;
                let from_impl_block_token_stream =
                    Self::from_impl_block_token_stream(&derive_input.ident, &type_chain)?;
                let from_impl_block_token_stream_iter = Some(from_impl_block_token_stream);

                from_impl_blocks_token_stream.extend(from_impl_block_token_stream_iter);
            }
        }

        Ok(from_impl_blocks_token_stream)
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let derive_input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&derive_input)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}
