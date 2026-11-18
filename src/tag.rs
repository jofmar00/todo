use crate::consts::{TAG_DEFAULT_NAME, TAG_LIGHTNESS, TAG_SATURATION};
use colored::{Colorize, CustomColor};
use colorsys::{Hsl, Rgb};
use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Tag(pub String);
impl Tag {
    pub fn color(&self) -> CustomColor {
        if self.0 == TAG_DEFAULT_NAME {
            return CustomColor::new(255, 255, 255);
        }

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

impl Default for Tag {
    fn default() -> Self {
        Tag(String::from(TAG_DEFAULT_NAME))
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("#{}", self.0).custom_color(self.color()).fmt(f)
    }
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Tag(String::from(value))
    }
}

impl FromStr for Tag {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.into())
    }
}
