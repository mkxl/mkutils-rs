use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenStream as TokenStream2};
use syn::{Data, DeriveInput, Field, Fields, Type};

pub struct Inner;

impl Inner {
    const fn get_type(field: &Field) -> &Type {
        &field.ty
    }

    fn method_block(input: &DeriveInput) -> TokenStream2 {
        let Data::Struct(data_struct) = &input.data else { return TokenStream2::new() };
        let Fields::Unnamed(fields_unnamed) = &data_struct.fields else { return TokenStream2::new() };
        let num_fields = fields_unnamed.unnamed.len();

        let (return_type, inner) = if num_fields == 1 {
            let return_type = &fields_unnamed.unnamed[0].ty;
            let return_type = quote::quote! { #return_type };
            let inner = quote::quote! { self.0 };

            (return_type, inner)
        } else {
            let field_index = (0..num_fields).map(Literal::usize_unsuffixed);
            let inner = quote::quote! { ( #(self.#field_index,)* ) };
            let return_type = fields_unnamed.unnamed.iter().map(Self::get_type);
            let return_type = quote::quote! { ( #(#return_type,)* ) };

            (return_type, inner)
        };

        quote::quote! {
            pub fn inner(&self) -> #return_type {
                #inner
            }
        }
    }

    pub fn derive(input_token_stream: TokenStream) -> TokenStream {
        let input = syn::parse_macro_input!(input_token_stream);
        let method_block = Self::method_block(&input);
        let input_ident = &input.ident;
        let (impl_generics, input_generics, input_where_clause) = input.generics.split_for_impl();
        let impl_block_token_stream = quote::quote! {
            impl #impl_generics #input_ident #input_generics #input_where_clause {
                #method_block
            }
        };

        impl_block_token_stream.into()
    }
}
