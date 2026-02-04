use crate::utils::Utils;
use derive_more::From;
use palette::Srgb;
use ratatui::style::Color;
use serde::{Deserialize, Deserializer, de::Error};
use std::str::FromStr;

type SrgbU8 = Srgb<u8>;

#[derive(From)]
pub struct Rgb {
    srgb: SrgbU8,
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
