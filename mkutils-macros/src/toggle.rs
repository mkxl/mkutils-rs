use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::{Data, DataEnum, DeriveInput, Error as SynError, Fields, Ident, Variant, spanned::Spanned};

pub struct Toggle;

impl Toggle {
    const NO_UNIT_VARIANTS_ERROR_MESSAGE: &'static str = "there are no unit variants on this enum";

    const fn unit_variant_ident(variant: &Variant) -> Option<&Ident> {
        if std::matches!(variant.fields, Fields::Unit) {
            Some(&variant.ident)
        } else {
            None
        }
    }

    fn no_unit_variants_syn_error(span: Span) -> SynError {
        SynError::new(span, Self::NO_UNIT_VARIANTS_ERROR_MESSAGE)
    }

    fn match_arms(data_enum: &DataEnum) -> Result<Vec<TokenStream2>, SynError> {
        // TODO-76211f
        let mut match_arms = Vec::new();
        let unit_variant_idents = data_enum
            .variants
            .iter()
            .filter_map(Self::unit_variant_ident)
            .collect::<Vec<&Ident>>();
        let num_unit_variants = unit_variant_idents.len();
        let mut index = 0;

        for from_variant in &data_enum.variants {
            let from_variant_ident = &from_variant.ident;
            let lhs = match &from_variant.fields {
                Fields::Unit => {
                    index += 1;

                    quote::quote! { Self::#from_variant_ident }
                }
                Fields::Named(_fields_named) => quote::quote! { Self::#from_variant_ident(..) },
                Fields::Unnamed(_fields_unnamed) => quote::quote! { Self::#from_variant_ident { .. } },
            };
            let Some(to_ident) = unit_variant_idents.get(index % num_unit_variants) else {
                return Err(Self::no_unit_variants_syn_error(data_enum.variants.span()));
            };
            let match_arm = quote::quote! { #lhs => Self::#to_ident, };

            match_arms.push(match_arm);
        }

        Ok(match_arms)
    }

    fn method_blocks(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let Data::Enum(data_enum) = &input.data else { return Ok(TokenStream2::new()) };
        let match_arms = Self::match_arms(data_enum)?;
        let method_blocks = quote::quote! {
            pub fn toggled(&self) -> Self {
                match self {
                    #(#match_arms)*
                }
            }

            pub fn toggle(&mut self) -> &mut Self {
                *self = self.toggled();

                self
            }
        };

        Ok(method_blocks)
    }

    fn derive_impl(input: &DeriveInput) -> Result<TokenStream2, SynError> {
        let method_blocks = Self::method_blocks(input)?;
        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #input_ident #input_generics #input_where_clause {
                #method_blocks
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
