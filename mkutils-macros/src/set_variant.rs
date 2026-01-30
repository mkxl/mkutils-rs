use heck::AsSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{Data, DeriveInput, Fields, Variant};

pub struct SetVariant;

impl SetVariant {
    fn method_block_from_variant(variant: &Variant) -> Option<TokenStream2> {
        // TODO-76211f: support empty tuple-struct and c-struct enum variants
        if !std::matches!(variant.fields, Fields::Unit) {
            return None;
        }

        let pascal_case_variant_ident = &variant.ident;
        let snake_case_variant_name = AsSnakeCase(pascal_case_variant_ident.to_string()).to_string();
        let method_ident = quote::format_ident!("set_{snake_case_variant_name}");
        let method_block = quote::quote! {
            pub fn #method_ident(&mut self) -> &mut Self {
                *self = Self::#pascal_case_variant_ident;

                self
            }
        };

        Some(method_block)
    }

    fn method_blocks(input: &DeriveInput) -> Vec<TokenStream2> {
        let Data::Enum(data_enum) = &input.data else { return Vec::new() };
        let method_block_iter = data_enum.variants.iter().filter_map(Self::method_block_from_variant);

        method_block_iter.collect()
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);
        let method_blocks = Self::method_blocks(&input);
        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #input_ident #input_generics #input_where_clause {
                #(#method_blocks)*
            }
        };

        impl_block_token_stream.into()
    }
}
