//! A thin abstraction over xproto::ModMask so it can be (de)serialized easily

use std::{
    convert::TryFrom,
    fmt::{self, Display},
    str::FromStr,
};

use {
    serde::{Deserialize, Serialize},
    x11rb::protocol::xproto::ModMask,
};

use common::ParseModMaskError;

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct XModMask(ModMask);

impl FromStr for XModMask {
    type Err = ParseModMaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "shift" => Ok(Self(ModMask::SHIFT)),
            "lock" => Ok(Self(ModMask::LOCK)),
            "control" | "ctrl" => Ok(Self(ModMask::CONTROL)),
            "mod1" | "mod 1" => Ok(Self(ModMask::M1)),
            "mod2" | "mod 2" => Ok(Self(ModMask::M2)),
            "mod3" | "mod 3" => Ok(Self(ModMask::M3)),
            "mod4" | "mod 4" => Ok(Self(ModMask::M4)),
            "mod5" | "mod 5" => Ok(Self(ModMask::M5)),
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
            ModMask::SHIFT => String::from("Shift"),
            ModMask::LOCK => String::from("Lock"),
            ModMask::CONTROL => String::from("Control"),
            ModMask::M1 => String::from("Mod 1"),
            ModMask::M2 => String::from("Mod 2"),
            ModMask::M3 => String::from("Mod 3"),
            ModMask::M4 => String::from("Mod 4"),
            ModMask::M5 => String::from("Mod 5"),
            ModMask::ANY => String::from("Any"),
            _ => unreachable!(),
        }
    }
}

impl Display for XModMask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl From<XModMask> for ModMask {
    fn from(mask: XModMask) -> Self {
        mask.0
    }
}

impl From<XModMask> for u16 {
    fn from(mask: XModMask) -> Self {
        u16::from(mask.0)
    }
}
