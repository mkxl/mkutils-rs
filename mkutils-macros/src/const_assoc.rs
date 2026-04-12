use crate::utils::Cat6;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    DeriveInput, Error as SynError, Expr, Ident, Token, Type, Visibility,
    parse::{Parse, ParseStream},
};

pub struct ConstAssoc {
    visibility: Visibility,
    ident: Ident,
    ty: Type,
    expr: Expr,
}

impl ConstAssoc {
    const ATTRIBUTE_NAME: &str = "const_assoc";

    fn from_derive_input(input: &DeriveInput) -> Result<Vec<Self>, SynError> {
        let mut const_assocs = Vec::new();

        for attribute in &input.attrs {
            if attribute.path().is_ident(Self::ATTRIBUTE_NAME) {
                const_assocs.push(attribute.parse_args::<Self>()?);
            }
        }

        Ok(const_assocs)
    }

    fn associated_item(&self) -> TokenStream2 {
        let Self {
            visibility,
            ident,
            ty,
            expr,
        } = self;

        quote::quote! {
            #visibility const #ident: #ty = #expr;
        }
    }

    fn derive_impl(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let DeriveInput { ident, generics, .. } = input;
        let const_assocs = Self::from_derive_input(input)?;
        let (impl_generics, input_generics, input_where_clause) = generics.split_for_impl();
        let associated_items = const_assocs.iter().map(Self::associated_item);
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #ident #input_generics #input_where_clause {
                #(#associated_items)*
            }
        };

        Ok(impl_block_token_stream)
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&input)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}

impl Parse for ConstAssoc {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        let (visibility, ident, _colon, ty, _equal, expr) = parse_stream
            .parse::<Cat6<Visibility, Ident, Token![:], Type, Token![=], Expr>>()?
            .into_tuple();
        let const_assoc = Self {
            visibility,
            ident,
            ty,
            expr,
        };

        Ok(const_assoc)
    }
}
