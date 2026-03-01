use crate::{error::Error, utils::CommaPunctuated};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, Error as SynError, Type, spanned::Spanned};

pub struct FromChain;

impl FromChain {
    const ATTRIBUTE_NAME: &str = "from_chain";

    fn from_impl_block_token_stream(
        input: &DeriveInput,
        mut type_chain: CommaPunctuated<Type>,
    ) -> Result<TokenStream2, SynError> {
        let self_type = syn::parse_quote!(Self);

        type_chain.push(self_type);

        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let Some(head_type) = type_chain.first() else {
            Err(Error::empty_attribute(type_chain.span(), Self::ATTRIBUTE_NAME))?
        };
        let head_ident = quote::format_ident!("value");
        let mut value = quote::quote! { #head_ident };
        let src_and_dst_type_pair_iter = type_chain.iter().rev().skip(1).zip(type_chain.iter().rev()).rev();

        for (src_type, dst_type) in src_and_dst_type_pair_iter {
            value = quote::quote! {
                <#dst_type as ::std::convert::From<#src_type>>::from(#value)
            };
        }

        let from_impl_block_token_stream = quote::quote! {
            impl #impl_generics ::std::convert::From<#head_type> for #input_ident #input_generics #input_where_clause {
                fn from(#head_ident: #head_type) -> Self {
                    #value
                }
            }
        };

        Ok(from_impl_block_token_stream)
    }

    #[allow(clippy::similar_names)]
    pub fn derive_impl(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let mut from_impl_blocks_token_stream = TokenStream2::new();

        for attribute in &input.attrs {
            if attribute.path().is_ident(Self::ATTRIBUTE_NAME) {
                let type_chain = attribute.parse_args_with(CommaPunctuated::<Type>::parse_terminated)?;
                let from_impl_block_token_stream = Self::from_impl_block_token_stream(input, type_chain)?;
                let from_impl_block_token_stream_iter = Some(from_impl_block_token_stream);

                from_impl_blocks_token_stream.extend(from_impl_block_token_stream_iter);
            }
        }

        Ok(from_impl_blocks_token_stream)
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&input)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}
