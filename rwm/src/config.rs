use std::{convert::TryFrom, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{layouts::LayoutType, mod_mask::XModMask};
use common::LoadConfigError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) border_width: u32,
    /// ARGB format
    pub(crate) focused_border_color: u32,
    pub(crate) normal_border_color: u32,
    pub(crate) mod_key: XModMask,
    /// First one is the default
    pub(crate) layouts: Vec<LayoutType>,
    /// If the focus will follow the cursor or not
    pub(crate) follow_cursor: bool,
    /// The path to the currently loaded config file.
    /// None if there is no config loaded
    #[serde(skip)]
    pub(crate) path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let blue_bytes = [
            255_u8, // Alpha
            000_u8, // Red
            000_u8, // Green
            255_u8, // Blue
        ];
        let blue = u32::from_be_bytes(blue_bytes);

        let gray_bytes = [
            255_u8, // Alpha
            211_u8, // Red
            211_u8, // Green
            211_u8, // Blue
        ];
        let gray = u32::from_be_bytes(gray_bytes);

        let mod_key = XModMask::try_from(String::from("mod1")).unwrap(); // left alt

        Self {
            border_width: 4, // pixels
            focused_border_color: blue,
            normal_border_color: gray,
            mod_key,
            layouts: vec![
                LayoutType::MonadTall,
                LayoutType::Grid,
                LayoutType::Floating,
            ],
            follow_cursor: true,
            path: None,
        }
    }
}

impl Config {
    pub(crate) fn load(&mut self, path: Option<PathBuf>) -> Result<(), LoadConfigError> {
        if path.is_none() && self.path.is_none() {
            return Err(LoadConfigError::new("No configuration file found"));
        }
        let path = path.unwrap_or_else(|| self.path.clone().unwrap());

        let new_config = fs::read_to_string(path.clone())?;
        let mut new_config: Self = toml::from_str(new_config.as_str())?;
        new_config.path = Some(path);

        std::mem::swap(self, &mut new_config);
        Ok(())
    }
}
