//! A struct that allows good and more correct (de)serialisation for colors.
//! Internally they need to be displayed in ARGB format but we want to parse and show them in hex

use std::{
    convert::TryFrom,
    fmt::{self, Display},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use common::ParseColorError;

const ALPHA: u8 = 255;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            red: r,
            green: g,
            blue: b,
        }
    }

    pub fn blue() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 255,
        }
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We assume that we are parsing hex. If it's not valid we error
        let s = s
            .trim_start_matches(|c: char| !c.is_ascii_hexdigit())
            .trim_end();
        if s.len() != 6 {
            return Err(ParseColorError {
                color: s.to_string(),
            });
        }
        let (r, s) = s.split_at(2);
        let red = u8::from_str_radix(r, 16)?;

        let (g, b) = s.split_at(2);
        let green = u8::from_str_radix(g, 16)?;

        let blue = u8::from_str_radix(b, 16)?;

        Ok(Self { red, green, blue })
    }
}

impl TryFrom<String> for Color {
    type Error = ParseColorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl From<Color> for String {
    fn from(c: Color) -> Self {
        format!("#{:02X}{:02X}{:02X}", c.red, c.green, c.blue)
    }
}

impl From<Color> for u32 {
    fn from(c: Color) -> Self {
        Self::from_be_bytes([ALPHA, c.red, c.green, c.blue])
    }
}

impl From<Color> for Option<u32> {
    fn from(c: Color) -> Self {
        Some(u32::from(c))
    }
}
