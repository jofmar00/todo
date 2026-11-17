use crate::consts::{TAG_LIGHTNESS, TAG_SATURATION};
use colored::{CustomColor, Colorize};
use colorsys::{Hsl, Rgb};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible, fmt::Display, hash::{DefaultHasher, Hash, Hasher}, str::FromStr,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag(pub String);
impl Tag {
    pub fn color(&self) -> CustomColor {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        let hue = (hasher.finish() % 360) as f64;
        let rgb: Rgb = Hsl::new(hue, TAG_SATURATION, TAG_LIGHTNESS, None).into();
        CustomColor::new(
            rgb.red().round() as u8,
            rgb.green().round() as u8,
            rgb.blue().round() as u8,
        )
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("#{}", self.0).custom_color(self.color()).fmt(f)
    }
}

impl FromStr for Tag {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tag(s.to_owned()))
    }
}