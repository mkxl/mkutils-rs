use crate::{error::Error, utils::CommaPunctuated};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Data, DeriveInput, Error as SynError, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, ImplItem,
    Index, Lit, Meta, Path, WherePredicate, spanned::Spanned,
};

pub struct Basic;

impl Basic {
    const ASSOCIATED_ITEM_ATTRIBUTE_NAME: &str = "assoc";
    const BOUND_ATTRIBUTE_NAME: &str = "bound";

    fn field_assignment(field: &Field, trait_path: &Path, method: &Ident) -> Result<TokenStream2, SynError> {
        let Some(field_name) = &field.ident else {
            return Err(Error::c_struct_field_missing_name(field));
        };
        let field_assignment =
            quote::quote! { #field_name: #trait_path::#method(&self.#field_name, &other.#field_name) };

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
            .map(|index| quote::quote! { #trait_path::#method(&self.#index, &other.#index) });

        quote::quote! { Self(#(#field_values),*) }
    }

    fn value_for_unit_struct() -> TokenStream2 {
        quote::quote! { Self }
    }

    fn value(input: &DeriveInput, trait_path: &Path, method: &Ident) -> Result<TokenStream2, SynError> {
        let Data::Struct(data_struct) = &input.data else {
            return Err(Error::unsupported_item_type(input));
        };
        let value = match &data_struct.fields {
            Fields::Named(fields_named) => Self::value_for_c_struct(fields_named, trait_path, method)?,
            Fields::Unnamed(fields_unnamed) => Self::value_for_tuple_struct(fields_unnamed, trait_path, method),
            Fields::Unit => Self::value_for_unit_struct(),
        };

        Ok(value)
    }

    fn parse_associated_item(meta: &Meta) -> Result<ImplItem, SynError> {
        let meta_list = meta.require_list()?;
        let associated_item_token_stream = &meta_list.tokens;
        let associated_item_res = syn::parse2::<ImplItem>(associated_item_token_stream.clone());

        if associated_item_res.is_ok() {
            return associated_item_res;
        }

        let associated_item_token_stream = quote::quote! { #associated_item_token_stream; };

        syn::parse2::<ImplItem>(associated_item_token_stream)
    }

    fn parse_attribute_args(
        attribute_args: CommaPunctuated<Meta>,
        generics: &mut Generics,
        associated_items: &mut Vec<ImplItem>,
        use_default_bounds: &mut bool,
    ) -> Result<(), SynError> {
        for meta in attribute_args {
            if meta.path().is_ident(Self::ASSOCIATED_ITEM_ATTRIBUTE_NAME) {
                let associated_item = Self::parse_associated_item(&meta)?;

                associated_items.push(associated_item);
            } else if meta.path().is_ident(Self::BOUND_ATTRIBUTE_NAME) {
                let meta_name_value = meta.require_name_value()?;
                let bounds_str = crate::utils::parse!(Expr::Lit, &meta_name_value.value)?;
                let bounds_str = crate::utils::parse!(Lit::Str, &bounds_str.lit)?;
                let bounds = bounds_str.parse_with(CommaPunctuated::<WherePredicate>::parse_terminated)?;

                generics.make_where_clause().predicates.extend(bounds);

                *use_default_bounds = false;
            } else {
                let expected_values = &[Self::ASSOCIATED_ITEM_ATTRIBUTE_NAME, Self::BOUND_ATTRIBUTE_NAME];
                let error = Error::unexpected_value_multi(&meta, expected_values);

                return Err(error);
            }
        }

        Ok(())
    }

    fn get_generics_and_associated_items(
        input: &DeriveInput,
        trait_path: &Path,
        attribute_name: &str,
    ) -> Result<(Generics, Vec<ImplItem>), SynError> {
        let mut generics = input.generics.clone();
        let mut associated_items = Vec::new();
        let mut use_default_bounds = true;

        for attribute in &input.attrs {
            if !attribute.path().is_ident(attribute_name) {
                continue;
            }

            let attribute_args = attribute.parse_args_with(CommaPunctuated::<Meta>::parse_terminated)?;

            Self::parse_attribute_args(
                attribute_args,
                &mut generics,
                &mut associated_items,
                &mut use_default_bounds,
            )?;
        }

        if use_default_bounds {
            let mut default_bounds = Vec::new();

            for type_param in generics.type_params() {
                let type_param_ident = &type_param.ident;
                let default_bound = quote::quote! { #type_param_ident: #trait_path };
                let default_bound = syn::parse2::<WherePredicate>(default_bound)?;

                default_bounds.push(default_bound);
            }

            generics.make_where_clause().predicates.extend(default_bounds);
        }

        Ok((generics, associated_items))
    }

    fn derive_impl(
        input: &DeriveInput,
        trait_path: &str,
        method: &str,
        other_type: &str,
        attribute_name: &str,
    ) -> Result<TokenStream2, SynError> {
        let input_ident = &input.ident;
        let trait_path = syn::parse_str::<Path>(trait_path)?;
        let (generics, associated_items) = Self::get_generics_and_associated_items(input, &trait_path, attribute_name)?;
        let (impl_generics, input_generics, input_where_clause) = generics.split_for_impl();
        let method = syn::parse_str::<Ident>(method)?;
        let other_type = syn::parse_str::<Path>(other_type)?;
        let value = Self::value(input, &trait_path, &method)?;
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #trait_path for #input_ident #input_generics #input_where_clause {
                #(#associated_items)*

                fn #method(&self, other: &#other_type) -> Self {
                    #value
                }
            }
        };

        Ok(impl_block_token_stream)
    }

    pub fn derive(
        input_token_stream: TokenStream,
        trait_path: &str,
        method: &str,
        other_type: &str,
        attribute_name: &str,
    ) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&input, trait_path, method, other_type, attribute_name)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}
