use crate::utils::Utils;
use derive_more::From;
use palette::{Lighten, Okhsl, Srgb};
use ratatui::style::Color;
use serde::{Deserialize, Deserializer, de::Error};
use std::str::FromStr;

type SrgbU8 = Srgb<u8>;

#[derive(Clone, Copy, Debug, From)]
pub struct Rgb {
    srgb: SrgbU8,
}

impl Rgb {
    #[must_use]
    pub fn lighten_fixed(&self, amount: f32) -> Self {
        self.srgb
            .into_format::<f32>()
            .into_color::<Okhsl<f32>>()
            .lighten_fixed(amount)
            .into_color::<Srgb<f32>>()
            .into_format()
            .into()
    }
}

impl FromStr for Rgb {
    type Err = <SrgbU8 as FromStr>::Err;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        hex.parse::<SrgbU8>()?.convert::<Self>().ok()
    }
}

impl From<Rgb> for Color {
    fn from(rgb: Rgb) -> Self {
        Self::Rgb(rgb.srgb.red, rgb.srgb.green, rgb.srgb.blue)
    }
}

impl<'a> Deserialize<'a> for Rgb {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        <&str>::deserialize(deserializer)?
            .parse::<Self>()
            .map_err(D::Error::custom)
    }
}
