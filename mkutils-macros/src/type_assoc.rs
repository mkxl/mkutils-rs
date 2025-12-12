use crate::utils::{Cat3, Comma, CommaPunctuated, IdentAssignment};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{
    DeriveInput, Error as SynError, Path, Type,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

pub struct TypeAssoc {
    trait_path: Path,
    assoc_type_token_streams: Vec<TokenStream2>,
}

impl TypeAssoc {
    const TYPE_ASSOC_ATTRIBUTE_NAME: &str = "type_assoc";
    const TRAIT_IDENT_NAME: &str = "trait";

    fn missing_type_assoc_attribute_error_message(span: Span) -> SynError {
        let message = std::format!(
            "no `{attribute_name}` attribute found",
            attribute_name = Self::TYPE_ASSOC_ATTRIBUTE_NAME
        );

        SynError::new(span, message)
    }

    fn unexpected_ident(span: Span) -> SynError {
        let message = std::format!(
            "expected `{trait_ident_name}` here",
            trait_ident_name = Self::TRAIT_IDENT_NAME
        );

        SynError::new(span, message)
    }

    fn assoc_type_token_stream(Cat3(assoc_type_ident, _comma, assoc_type_type): IdentAssignment<Type>) -> TokenStream2 {
        quote::quote! {
            type #assoc_type_ident = #assoc_type_type;
        }
    }

    fn from_derive_input(derive_input: &DeriveInput) -> Result<Self, SynError> {
        for attribute in &derive_input.attrs {
            if attribute.path().is_ident(Self::TYPE_ASSOC_ATTRIBUTE_NAME) {
                return attribute.parse_args();
            }
        }

        Err(Self::missing_type_assoc_attribute_error_message(derive_input.span()))
    }

    pub fn derive_impl(derive_input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let derive_input_ident = &derive_input.ident;
        let Self {
            trait_path,
            assoc_type_token_streams,
        } = Self::from_derive_input(derive_input)?;
        let (impl_generics, input_generics, input_where_clause) = derive_input.generics.split_for_impl();
        let impl_block_tokens = quote::quote! {
            impl #impl_generics #trait_path for #derive_input_ident #input_generics #input_where_clause {
                #(#assoc_type_token_streams)*
            }
        };

        Ok(impl_block_tokens)
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let derive_input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&derive_input)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}

impl Parse for TypeAssoc {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        let (trait_ident, _equals, trait_path) = parse_stream.parse::<IdentAssignment<Path>>()?.into_tuple();

        if trait_ident != Self::TRAIT_IDENT_NAME {
            return Err(Self::unexpected_ident(trait_ident.span()));
        }

        parse_stream.parse::<Comma>()?;

        let assoc_type_assignments = CommaPunctuated::<IdentAssignment<Type>>::parse_terminated(parse_stream)?;
        let assoc_type_token_streams = assoc_type_assignments
            .into_iter()
            .map(Self::assoc_type_token_stream)
            .collect();
        let type_assoc = Self {
            trait_path,
            assoc_type_token_streams,
        };

        Ok(type_assoc)
    }
}
