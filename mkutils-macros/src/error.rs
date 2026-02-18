use proc_macro2::Span;
use syn::Error as SynError;

pub struct Error;

impl Error {
    const C_STRUCT_FIELD_MISSING_NAME_ERROR_MESSAGE: &str =
        "a field in this C struct unexpectedly does not have a name";
    const NO_UNIT_VARIANTS_ERROR_MESSAGE: &'static str = "there are no unit variants on this enum";
    const UNSUPPORTED_ITEM_TYPE_ERROR_MESSAGE: &str = "this macro is currently only supported on struct types";

    pub fn unsupported_item_type(span: Span) -> SynError {
        SynError::new(span, Self::UNSUPPORTED_ITEM_TYPE_ERROR_MESSAGE)
    }

    pub fn c_struct_field_missing_name(span: Span) -> SynError {
        SynError::new(span, Self::C_STRUCT_FIELD_MISSING_NAME_ERROR_MESSAGE)
    }

    pub fn no_unit_variants_syn_error(span: Span) -> SynError {
        SynError::new(span, Self::NO_UNIT_VARIANTS_ERROR_MESSAGE)
    }

    pub fn missing_expected_attribute(span: Span, attribute_name: &str) -> SynError {
        let message = std::format!("no `{attribute_name}` attribute found");

        SynError::new(span, message)
    }

    pub fn unexpected_value(span: Span, expected_value: &str) -> SynError {
        let message = std::format!("expected `{expected_value}` here");

        SynError::new(span, message)
    }

    pub fn empty_attribute(span: Span, attribute_name: &str) -> SynError {
        let message = std::format!("empty `{attribute_name}` attribute found");

        SynError::new(span, message)
    }
}
