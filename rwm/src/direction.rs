use std::str::FromStr;

use crate::errors::DirectionValueError;

#[derive(Debug)]
pub(crate) enum Direction {
    Up,
    Down,
}

impl FromStr for Direction {
    type Err = DirectionValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dir = s.trim().to_lowercase();
        if dir == "up" {
            Ok(Self::Up)
        } else if dir == "down" {
            Ok(Self::Down)
        } else {
            Err(DirectionValueError { msg: s.to_string() })
        }
    }
}
