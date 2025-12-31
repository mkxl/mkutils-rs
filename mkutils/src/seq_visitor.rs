use crate::utils::Utils;
use serde::{
    Deserialize,
    de::{Error, SeqAccess, Visitor},
};
use std::{
    fmt::{Display, Error as FmtError, Formatter},
    marker::PhantomData,
};

pub struct SeqVisitor<X, Y, C, E, F> {
    func: F,
    phantom: PhantomData<(X, Y, C, E)>,
}

impl<X, Y, C, E, F> SeqVisitor<X, Y, C, E, F> {
    pub const fn new(func: F) -> Self {
        let phantom = PhantomData;

        Self { func, phantom }
    }
}

impl<'de, X: Deserialize<'de>, Y, C: Default + Extend<Y>, E: Display, F: Fn(X) -> Result<Y, E>> Visitor<'de>
    for SeqVisitor<X, Y, C, E, F>
{
    type Value = C;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
        std::write!(formatter, "a sequence of {element_type}", element_type = Y::type_name())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut collection = C::default();

        while let Some(item) = seq.next_element::<X>()? {
            let item = (self.func)(item).map_err(A::Error::custom)?;

            collection.extend(item.once());
        }

        collection.ok()
    }
}
