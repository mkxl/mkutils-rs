use crate::utils::Utils;
use serde_json::{Map, Value as Json};
use valuable::{Listable, Mappable, Valuable, Value, Visit};

#[repr(transparent)]
pub struct ValuableJson(Json);

#[repr(transparent)]
pub struct ValuableJsonArray(Vec<Json>);

#[repr(transparent)]
pub struct ValuableJsonObject(Map<String, Json>);

impl Valuable for ValuableJson {
    #[allow(clippy::option_if_let_else)]
    fn as_value(&self) -> Value<'_> {
        match &self.0 {
            Json::Null => Value::Unit,
            Json::Bool(boolean) => boolean.as_value(),
            Json::Number(number) => {
                // NOTE: intentionally check i128 and u128 before f64
                if let Some(number) = number.as_i128() {
                    Value::I128(number)
                } else if let Some(number) = number.as_u128() {
                    Value::U128(number)
                } else if let Some(number) = number.as_f64() {
                    Value::F64(number)
                } else {
                    f64::NAN.as_value()
                }
            }
            Json::String(string) => string.as_value(),
            Json::Array(array) => array.cast_ref::<ValuableJsonArray>().as_value(),
            Json::Object(map) => map.cast_ref::<ValuableJsonObject>().as_value(),
        }
    }

    fn visit(&self, visit: &mut dyn Visit) {
        self.as_value().visit(visit);
    }
}

impl Valuable for ValuableJsonArray {
    fn as_value(&self) -> Value<'_> {
        Value::Listable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        for value in &self.0 {
            value.cast_ref::<ValuableJson>().visit(visit);
        }
    }
}

impl Listable for ValuableJsonArray {
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.iter().size_hint()
    }
}

impl Valuable for ValuableJsonObject {
    fn as_value(&self) -> Value<'_> {
        Value::Mappable(self)
    }

    fn visit(&self, visit: &mut dyn Visit) {
        for (key, value) in &self.0 {
            visit.visit_entry(key.as_value(), value.cast_ref::<ValuableJson>().as_value());
        }
    }
}

impl Mappable for ValuableJsonObject {
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.iter().size_hint()
    }
}

pub trait AsValuable {
    fn as_valuable(&self) -> Value<'_>;
}

impl AsValuable for Json {
    fn as_valuable(&self) -> Value<'_> {
        self.cast_ref::<ValuableJson>().as_value()
    }
}
