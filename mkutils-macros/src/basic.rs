use crate::error::Error;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Data, DeriveInput, Error as SynError, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Index, Path,
    spanned::Spanned,
};

pub struct Basic;

impl Basic {
    fn field_assignment(field: &Field, trait_path: &Path, method: &Ident) -> Result<TokenStream2, SynError> {
        let Some(field_name) = &field.ident else {
            return Err(Error::c_struct_field_missing_name(field.span()));
        };
        let field_assignment =
            quote::quote! { #field_name: <Self as #trait_path>::#method(&self.#field_name, &other.#field_name) };

        Ok(field_assignment)
    }

    fn value_for_c_struct(
        fields_named: &FieldsNamed,
        trait_path: &Path,
        method: &Ident,
    ) -> Result<TokenStream2, SynError> {
        let field_assignments = fields_named
            .named
            .iter()
            .map(|field| Self::field_assignment(field, trait_path, method))
            .collect::<Result<Vec<TokenStream2>, SynError>>()?;
        let value = quote::quote! { Self { #(#field_assignments),* } };

        Ok(value)
    }

    fn value_for_tuple_struct(fields_unnamed: &FieldsUnnamed, trait_path: &Path, method: &Ident) -> TokenStream2 {
        let num_fields = fields_unnamed.unnamed.len();
        let field_values = (0..num_fields)
            .map(Index::from)
            .map(|index| quote::quote! { <Self as #trait_path>::#method(&self.#index, &other.#index) });

        quote::quote! { Self(#(#field_values),*) }
    }

    fn value_for_unit_struct() -> TokenStream2 {
        quote::quote! { Self }
    }

    fn value(input: &DeriveInput, trait_path: &Path, method: &Ident) -> Result<TokenStream2, SynError> {
        let Data::Struct(data_struct) = &input.data else {
            return Err(Error::unsupported_item_type(input.span()));
        };
        let value = match &data_struct.fields {
            Fields::Named(fields_named) => Self::value_for_c_struct(fields_named, trait_path, method)?,
            Fields::Unnamed(fields_unnamed) => Self::value_for_tuple_struct(fields_unnamed, trait_path, method),
            Fields::Unit => Self::value_for_unit_struct(),
        };

        Ok(value)
    }

    fn derive_impl(input: &DeriveInput, trait_path: &str, method: &str) -> Result<TokenStream2, SynError> {
        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let trait_path = syn::parse_str::<Path>(trait_path)?;
        let method = syn::parse_str::<Ident>(method)?;
        let value = Self::value(input, &trait_path, &method)?;
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #trait_path for #input_ident #input_generics #input_where_clause {
                fn #method(&self, other: &Self) -> Self {
                    #value
                }
            }
        };

        Ok(impl_block_token_stream)
    }

    pub fn derive(input_token_stream: TokenStream, trait_path: &str, method: &str) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&input, trait_path, method)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}
