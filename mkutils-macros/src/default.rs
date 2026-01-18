use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Data, DeriveInput, Error as SynError, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, spanned::Spanned};

pub struct Default;

impl Default {
    const ATTRIBUTE_NAME: &str = "default";
    const UNSUPPORTED_ITEM_TYPE_ERROR_MESSAGE: &str = "this macro is currently only supported on struct types";
    const C_STRUCT_FIELD_MISSING_NAME_ERROR_MESSAGE: &str =
        "a field in this C struct unexpectedly does not have a name";

    fn unsupported_item_type(span: Span) -> SynError {
        SynError::new(span, Self::UNSUPPORTED_ITEM_TYPE_ERROR_MESSAGE)
    }

    fn c_struct_field_missing_name(span: Span) -> SynError {
        SynError::new(span, Self::C_STRUCT_FIELD_MISSING_NAME_ERROR_MESSAGE)
    }

    fn std_default_field_value() -> TokenStream2 {
        quote::quote! { ::std::default::Default::default() }
    }

    fn default_field_value(field: &Field) -> Result<TokenStream2, SynError> {
        for attribute in &field.attrs {
            if attribute.path().is_ident(Self::ATTRIBUTE_NAME) {
                let default_field_value = attribute.parse_args::<Expr>()?;
                let default_field_value = quote::quote! { #default_field_value };

                return Ok(default_field_value);
            }
        }

        Ok(Self::std_default_field_value())
    }

    fn default_field_assignment(field: &Field) -> Result<TokenStream2, SynError> {
        let Some(field_name) = &field.ident else {
            return Err(Self::c_struct_field_missing_name(field.span()));
        };
        let default_field_value = Self::default_field_value(field)?;
        let default_field_assignment = quote::quote! { #field_name: #default_field_value };

        Ok(default_field_assignment)
    }

    fn default_value_for_c_struct(fields_named: &FieldsNamed) -> Result<TokenStream2, SynError> {
        let default_field_assignments = fields_named
            .named
            .iter()
            .map(Self::default_field_assignment)
            .collect::<Result<Vec<TokenStream2>, SynError>>()?;
        let default_value = quote::quote! { Self { #(#default_field_assignments),* } };

        Ok(default_value)
    }

    fn default_value_for_tuple_struct(fields_unnamed: &FieldsUnnamed) -> Result<TokenStream2, SynError> {
        let field_values = fields_unnamed
            .unnamed
            .iter()
            .map(Self::default_field_value)
            .collect::<Result<Vec<TokenStream2>, SynError>>()?;
        let default_value = quote::quote! { Self(#(#field_values),*) };

        Ok(default_value)
    }

    fn default_value_for_unit_struct() -> TokenStream2 {
        quote::quote! { Self }
    }

    fn default_value(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let Data::Struct(data_struct) = &input.data else {
            return Err(Self::unsupported_item_type(input.span()));
        };
        let default_value = match &data_struct.fields {
            Fields::Named(fields_named) => Self::default_value_for_c_struct(fields_named)?,
            Fields::Unnamed(fields_unnamed) => Self::default_value_for_tuple_struct(fields_unnamed)?,
            Fields::Unit => Self::default_value_for_unit_struct(),
        };

        Ok(default_value)
    }

    fn derive_impl(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let default_value = Self::default_value(input)?;
        let default_impl_block_token_stream = quote::quote! {
            impl #impl_generics ::std::default::Default for #input_ident #input_generics #input_where_clause {
                fn default() -> Self {
                    #default_value
                }
            }
        };

        Ok(default_impl_block_token_stream)
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&input)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}
