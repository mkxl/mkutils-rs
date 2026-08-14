use crate::error::Error;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    Error as SynError, Ident, ItemImpl, Path, Type, Visibility,
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

enum EmptyType {
    Enum,
    UnitStruct,
    CStruct,
}

impl EmptyType {
    fn get_declaration(&self, ident: &Ident) -> TokenStream2 {
        match self {
            Self::Enum => quote::quote!(enum #ident {}),
            Self::UnitStruct => quote::quote!(struct #ident;),
            Self::CStruct => quote::quote!(struct #ident {}),
        }
    }
}

pub struct Empty {
    visibility: Visibility,
    empty_type: EmptyType,
}

impl Parse for Empty {
    fn parse(parse_stream: ParseStream) -> Result<Self, SynError> {
        let visibility = parse_stream.parse()?;
        let empty_type = if parse_stream.is_empty() {
            EmptyType::Enum
        } else {
            let ident = Ident::parse_any(parse_stream)?;

            match ident.to_string().as_str() {
                "enum" => EmptyType::Enum,
                "unit_struct" => EmptyType::UnitStruct,
                "c_struct" => EmptyType::CStruct,
                _ => {
                    return Err(Error::unexpected_value_multi(
                        &ident,
                        &["enum", "unit_struct", "c_struct"],
                    ));
                }
            }
        };
        let empty = Self { visibility, empty_type };

        Ok(empty)
    }
}

impl Empty {
    fn get_ident(item_impl: &ItemImpl) -> Result<&Ident, SynError> {
        let Type::Path(type_path) = item_impl.self_ty.as_ref() else {
            return Err(Error::unexpected_value(
                item_impl.self_ty.as_ref(),
                std::any::type_name::<Path>(),
            ));
        };
        let Some(segment) = type_path.path.segments.last() else {
            return Err(Error::empty_path(&type_path.path));
        };

        Ok(&segment.ident)
    }

    fn derive_impl(&self, item_impl: &mut ItemImpl) -> Result<TokenStream2, SynError> {
        let Self { visibility, empty_type } = self;
        let attributes = std::mem::take(&mut item_impl.attrs);
        let ident = Self::get_ident(item_impl)?;
        let declaration = empty_type.get_declaration(ident);
        let declaration_and_impl_token_stream = quote::quote! {
            #(#attributes)*
            #visibility #declaration

            #item_impl
        };

        Ok(declaration_and_impl_token_stream)
    }

    pub fn derive(attr_args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
        let empty = syn::parse_macro_input!(attr_args_token_stream as Self);
        let mut item_impl = syn::parse_macro_input!(input_token_stream);

        empty
            .derive_impl(&mut item_impl)
            .unwrap_or_else(SynError::into_compile_error)
            .into()
    }
}
