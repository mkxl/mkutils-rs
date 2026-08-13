use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::{FnArg, GenericParam, Ident, ImplItemFn, parse::Nothing, spanned::Spanned};

pub enum With {}

impl With {
    fn get_with_method_ident(original_method: &ImplItemFn) -> Ident {
        let original_method_ident_str = original_method.sig.ident.to_string();
        let with_method_ident_suffix = if let Some((_prefix, suffix)) = original_method_ident_str.split_once('_') {
            suffix
        } else {
            original_method_ident_str.as_str()
        };

        quote::format_ident!(
            "with_{with_method_ident_suffix}",
            span = original_method.sig.ident.span()
        )
    }

    const fn get_generic_argument_ident(parameter: &GenericParam) -> Option<&Ident> {
        match parameter {
            GenericParam::Type(parameter) => Some(&parameter.ident),
            GenericParam::Const(parameter) => Some(&parameter.ident),
            GenericParam::Lifetime(_) => None,
        }
    }

    fn get_original_method_application_turbofish(original_method: &ImplItemFn) -> TokenStream2 {
        let mut generic_argument_idents = original_method
            .sig
            .generics
            .params
            .iter()
            .filter_map(Self::get_generic_argument_ident)
            .peekable();

        if generic_argument_idents.peek().is_none() {
            TokenStream2::new()
        } else {
            quote::quote!(::<#(#generic_argument_idents),*>)
        }
    }

    fn get_with_method_parameter_idents_and_is_method(with_method: &mut ImplItemFn) -> (Vec<Ident>, bool) {
        let mut with_method_parameter_idents = Vec::new();
        let mut is_method = false;

        for (index, argument) in with_method.sig.inputs.iter_mut().enumerate() {
            match argument {
                FnArg::Receiver(receiver) => {
                    is_method = true;
                    *receiver = if receiver.mutability.is_some() {
                        syn::parse_quote!(mut self)
                    } else {
                        syn::parse_quote!(self)
                    }
                }
                FnArg::Typed(pat_type) => {
                    let with_method_parameter_ident =
                        quote::format_ident!("parameter_{index}", span = pat_type.pat.span());

                    *pat_type.pat = syn::parse_quote!(#with_method_parameter_ident);

                    with_method_parameter_idents.push(with_method_parameter_ident);
                }
            }
        }

        (with_method_parameter_idents, is_method)
    }

    fn get_original_method_application(original_method: &ImplItemFn, with_method: &mut ImplItemFn) -> TokenStream2 {
        let original_method_application_turbofish = Self::get_original_method_application_turbofish(original_method);
        let (with_method_parameter_idents, is_method) =
            Self::get_with_method_parameter_idents_and_is_method(with_method);
        let method_caller = if is_method {
            quote::quote!(self.)
        } else {
            quote::quote!(Self::)
        };
        let original_method_ident = &original_method.sig.ident;
        let mut original_method_application = quote::quote!(
            #method_caller #original_method_ident #original_method_application_turbofish (#(#with_method_parameter_idents),*)
        );

        if with_method.sig.unsafety.is_some() {
            original_method_application = quote::quote!(unsafe { #original_method_application });
        }

        if with_method.sig.asyncness.is_some() {
            original_method_application = quote::quote!((#original_method_application).await);
        }

        original_method_application
    }

    fn derive_impl(original_method: &ImplItemFn) -> TokenStream2 {
        let mut with_method = original_method.clone();
        let original_method_application = Self::get_original_method_application(original_method, &mut with_method);

        with_method.sig.ident = Self::get_with_method_ident(original_method);
        with_method.sig.output = syn::parse_quote!(-> Self);
        with_method.block = syn::parse_quote!({
            let _ = #original_method_application;

            self
        });

        quote::quote! {
            #original_method

            #with_method
        }
    }

    pub fn derive(attr_args_token_stream: TokenStream, input_token_stream: TokenStream) -> TokenStream {
        let Nothing = syn::parse_macro_input!(attr_args_token_stream);
        let original_method = syn::parse_macro_input!(input_token_stream);

        Self::derive_impl(&original_method).into()
    }
}
