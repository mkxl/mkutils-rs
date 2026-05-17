pub trait SaturatingAddSigned {
    type Signed;

    #[must_use]
    fn saturating_add_signed(&self, other: &Self::Signed) -> Self;
}

macro_rules! impl_saturating_add_signed {
 ($type:ty, $signed_type:ty) => {
     impl SaturatingAddSigned for $type {
         type Signed = $signed_type;

         fn saturating_add_signed(&self, other: &Self::Signed) -> Self {
             Self::saturating_add_signed(*self, *other)
         }
     }
 };
 ($($suffix:tt),*) => {
     $(
         ::paste::paste! {
             impl_saturating_add_signed!([<u $suffix>], [<i $suffix>]);
         }
     )+
 };
}

impl_saturating_add_signed!(8, 16, 32, 64, 128, size);
