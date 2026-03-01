use crate::error::Error;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Data, DeriveInput, Error as SynError, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Visibility,
    spanned::Spanned,
};

pub struct Constructor;

impl Constructor {
    const ATTRIBUTE_NAME: &str = "new";

    fn visibility(input: &DeriveInput) -> Result<Visibility, SynError> {
        for attribute in &input.attrs {
            if attribute.path().is_ident(Self::ATTRIBUTE_NAME) {
                return attribute.parse_args::<Visibility>();
            }
        }

        Ok(Visibility::Inherited)
    }

    fn ident_field_pair(field: &Field) -> Result<(&Ident, &Field), SynError> {
        let Some(ident) = &field.ident else {
            return Err(Error::c_struct_field_missing_name(field.span()));
        };
        let ident_field_pair = (ident, field);

        Ok(ident_field_pair)
    }

    fn constructor_method_block_for_c_struct(
        fields_named: &FieldsNamed,
        visibility: &Visibility,
    ) -> Result<TokenStream2, SynError> {
        let ident_field_pairs = fields_named
            .named
            .iter()
            .map(Self::ident_field_pair)
            .collect::<Result<Vec<(&Ident, &Field)>, SynError>>()?;
        let mut constructor_parameter_idents = Vec::new();
        let mut constructor_parameters = Vec::new();

        for (constructor_parameter_ident, field) in ident_field_pairs {
            let field_type = &field.ty;
            let constructor_parameter = quote::quote! { #constructor_parameter_ident: #field_type };

            constructor_parameter_idents.push(constructor_parameter_ident);
            constructor_parameters.push(constructor_parameter);
        }

        let constructor_method_block = quote::quote! {
            #visibility fn new(#(#constructor_parameters),*) -> Self {
                Self { #(#constructor_parameter_idents),* }
            }
        };

        Ok(constructor_method_block)
    }

    fn constructor_method_block_for_tuple_struct(
        fields_unnamed: &FieldsUnnamed,
        visibility: &Visibility,
    ) -> TokenStream2 {
        let mut constructor_parameter_idents = Vec::new();
        let mut constructor_parameters = Vec::new();

        for (index, field) in fields_unnamed.unnamed.iter().enumerate() {
            let field_type = &field.ty;
            let constructor_parameter_ident = quote::format_ident!("field_{index}");
            let constructor_parameter = quote::quote! { #constructor_parameter_ident: #field_type };

            constructor_parameter_idents.push(constructor_parameter_ident);
            constructor_parameters.push(constructor_parameter);
        }

        quote::quote! {
            #visibility fn new(#(#constructor_parameters),*) -> Self {
                Self(#(#constructor_parameter_idents),*)
            }
        }
    }

    fn constructor_method_block_for_unit_struct(visibility: &Visibility) -> TokenStream2 {
        quote::quote! {
            #visibility fn new() -> Self {
                Self
            }
        }
    }

    fn constructor_method_block(input: &DeriveInput, visibility: &Visibility) -> Result<TokenStream2, SynError> {
        let Data::Struct(data_struct) = &input.data else {
            return Err(Error::unsupported_item_type(input.span()));
        };
        let constructor_method_block = match &data_struct.fields {
            Fields::Named(fields_named) => Self::constructor_method_block_for_c_struct(fields_named, visibility)?,
            Fields::Unnamed(fields_unnamed) => {
                Self::constructor_method_block_for_tuple_struct(fields_unnamed, visibility)
            }
            Fields::Unit => Self::constructor_method_block_for_unit_struct(visibility),
        };

        Ok(constructor_method_block)
    }

    fn derive_impl(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let visibility = Self::visibility(input)?;
        let constructor_method_block = Self::constructor_method_block(input, &visibility)?;
        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #input_ident #input_generics #input_where_clause {
                #constructor_method_block
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
