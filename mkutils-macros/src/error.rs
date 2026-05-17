use syn::{Error as SynError, spanned::Spanned};

pub struct Error;

impl Error {
    const C_STRUCT_FIELD_MISSING_NAME_ERROR_MESSAGE: &str =
        "a field in this C struct unexpectedly does not have a name";
    const NO_UNIT_VARIANTS_ERROR_MESSAGE: &str = "there are no unit variants on this enum";
    const UNSUPPORTED_ITEM_TYPE_ERROR_MESSAGE: &str = "this macro is currently only supported on struct types";

    pub fn unsupported_item_type<T: Spanned>(spanned: &T) -> SynError {
        SynError::new(spanned.span(), Self::UNSUPPORTED_ITEM_TYPE_ERROR_MESSAGE)
    }

    pub fn c_struct_field_missing_name<T: Spanned>(spanned: &T) -> SynError {
        SynError::new(spanned.span(), Self::C_STRUCT_FIELD_MISSING_NAME_ERROR_MESSAGE)
    }

    pub fn no_unit_variants<T: Spanned>(spanned: &T) -> SynError {
        SynError::new(spanned.span(), Self::NO_UNIT_VARIANTS_ERROR_MESSAGE)
    }

    pub fn missing_expected_attribute<T: Spanned>(spanned: &T, attribute_name: &str) -> SynError {
        let message = std::format!("no `{attribute_name}` attribute found");

        SynError::new(spanned.span(), message)
    }

    pub fn unexpected_value<T: Spanned>(spanned: &T, expected_value: &str) -> SynError {
        Self::unexpected_value_multi(spanned, &[expected_value])
    }

    fn quote(text: &str) -> String {
        std::format!("`{text}`")
    }

    pub fn unexpected_value_multi<T: Spanned>(spanned: &T, expected_values: &[&str]) -> SynError {
        let expected = if let Some((last, prefix)) = expected_values.split_last() {
            if prefix.is_empty() {
                Self::quote(last)
            } else {
                let last = Self::quote(last);
                let prefix = prefix
                    .iter()
                    .copied()
                    .map(Self::quote)
                    .collect::<Vec<String>>()
                    .join(", ");

                std::format!("{prefix}, or {last}")
            }
        } else {
            "nothing".to_owned()
        };
        let error_message = std::format!("expected {expected} here");

        SynError::new(spanned.span(), error_message)
    }

    pub fn empty_attribute<T: Spanned>(spanned: &T, attribute_name: &str) -> SynError {
        let message = std::format!("empty `{attribute_name}` attribute found");

        SynError::new(spanned.span(), message)
    }
}
