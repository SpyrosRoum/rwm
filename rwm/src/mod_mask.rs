//! A thin abstraction over xproto::ModMask so it can be (de)serialized easily

use std::{
    convert::TryFrom,
    fmt::{self, Display},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use x11rb::protocol::xproto::ModMask;

use common::ParseModMaskError;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct XModMask(ModMask);

impl FromStr for XModMask {
    type Err = ParseModMaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "shift" => Ok(Self(ModMask::Shift)),
            "lock" => Ok(Self(ModMask::Lock)),
            "control" | "ctrl" => Ok(Self(ModMask::Control)),
            "mod1" => Ok(Self(ModMask::M1)),
            "mod2" => Ok(Self(ModMask::M2)),
            "mod3" => Ok(Self(ModMask::M3)),
            "mod4" => Ok(Self(ModMask::M4)),
            "mod5" => Ok(Self(ModMask::M5)),
            _ => Err(ParseModMaskError {
                mask: s.to_string(),
            }),
        }
    }
}

impl TryFrom<String> for XModMask {
    type Error = ParseModMaskError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<XModMask> for String {
    fn from(mask: XModMask) -> Self {
        match mask.0 {
            ModMask::Shift => String::from("Shift"),
            ModMask::Lock => String::from("Lock"),
            ModMask::Control => String::from("Control"),
            ModMask::M1 => String::from("Mod 1"),
            ModMask::M2 => String::from("Mod 2"),
            ModMask::M3 => String::from("Mod 3"),
            ModMask::M4 => String::from("Mod 4"),
            ModMask::M5 => String::from("Mod 5"),
            ModMask::Any => String::from("Any"),
            _ => unreachable!(),
        }
    }
}

impl Display for XModMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = String::try_from(*self).unwrap();
        write!(f, "{}", v)
    }
}

impl From<XModMask> for ModMask {
    fn from(mask: XModMask) -> Self {
        mask.0
    }
}

impl From<XModMask> for u16 {
    fn from(mask: XModMask) -> Self {
        mask.0 as u16
    }
}
