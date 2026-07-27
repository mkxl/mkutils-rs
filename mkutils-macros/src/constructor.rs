use crate::error::Error;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Data, DeriveInput, Error as SynError, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Visibility,
    parse::{Parse, ParseStream},
};

struct ConstructorArgs {
    visibility: Visibility,
    name: Ident,
}

impl ConstructorArgs {
    const DEFAULT_VISIBILITY: Visibility = Visibility::Inherited;
    const DEFAULT_NAME: &str = "new";

    fn default_name() -> Ident {
        quote::format_ident!("{}", Self::DEFAULT_NAME)
    }
}

impl Default for ConstructorArgs {
    fn default() -> Self {
        let visibility = Self::DEFAULT_VISIBILITY;
        let name = Self::default_name();

        Self { visibility, name }
    }
}

impl Parse for ConstructorArgs {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        let visibility = parse_stream.parse()?;
        let name = if parse_stream.is_empty() {
            Self::default_name()
        } else {
            parse_stream.parse()?
        };
        let constructor_args = Self { visibility, name };

        Ok(constructor_args)
    }
}

pub struct Constructor;

impl Constructor {
    const ATTRIBUTE_NAME: &str = "constructor";

    fn get_constructor_args(input: &DeriveInput) -> Result<ConstructorArgs, SynError> {
        for attribute in &input.attrs {
            if attribute.path().is_ident(Self::ATTRIBUTE_NAME) {
                return attribute.parse_args();
            }
        }

        Ok(ConstructorArgs::default())
    }

    fn ident_field_pair(field: &Field) -> Result<(&Ident, &Field), SynError> {
        let Some(ident) = &field.ident else {
            return Err(Error::c_struct_field_missing_name(field));
        };
        let ident_field_pair = (ident, field);

        Ok(ident_field_pair)
    }

    fn constructor_method_block_for_c_struct(
        fields_named: &FieldsNamed,
        ConstructorArgs { visibility, name }: &ConstructorArgs,
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
            #visibility fn #name(#(#constructor_parameters),*) -> Self {
                Self { #(#constructor_parameter_idents),* }
            }
        };

        Ok(constructor_method_block)
    }

    fn constructor_method_block_for_tuple_struct(
        fields_unnamed: &FieldsUnnamed,
        ConstructorArgs { visibility, name }: &ConstructorArgs,
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
            #visibility fn #name(#(#constructor_parameters),*) -> Self {
                Self(#(#constructor_parameter_idents),*)
            }
        }
    }

    fn constructor_method_block_for_unit_struct(
        ConstructorArgs { visibility, name }: &ConstructorArgs,
    ) -> TokenStream2 {
        quote::quote! {
            #visibility fn #name() -> Self {
                Self
            }
        }
    }

    fn constructor_method_block(
        input: &DeriveInput,
        constructor_args: &ConstructorArgs,
    ) -> Result<TokenStream2, SynError> {
        let Data::Struct(data_struct) = &input.data else {
            return Err(Error::unsupported_item_type(input));
        };
        let constructor_method_block = match &data_struct.fields {
            Fields::Named(fields_named) => Self::constructor_method_block_for_c_struct(fields_named, constructor_args)?,
            Fields::Unnamed(fields_unnamed) => {
                Self::constructor_method_block_for_tuple_struct(fields_unnamed, constructor_args)
            }
            Fields::Unit => Self::constructor_method_block_for_unit_struct(constructor_args),
        };

        Ok(constructor_method_block)
    }

    fn derive_impl(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let constructor_args = Self::get_constructor_args(input)?;
        let constructor_method_block = Self::constructor_method_block(input, &constructor_args)?;
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
