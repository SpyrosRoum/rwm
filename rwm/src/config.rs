use std::{convert::TryFrom, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{color::Color, layouts::LayoutType, mod_mask::XModMask};
use common::LoadConfigError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) border_width: u32,
    pub(crate) focused_border_color: Color,
    pub(crate) normal_border_color: Color,
    pub(crate) mod_key: XModMask,
    /// First one is the default
    pub(crate) layouts: Vec<LayoutType>,
    /// If the focus will follow the cursor or not
    pub(crate) follow_cursor: bool,
    /// Useless gap between windows
    pub(crate) gap: u32,
    /// The path to the currently loaded config file.
    /// None if there is no config loaded
    #[serde(skip)]
    pub(crate) path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        let blue = Color::blue();
        let gray = Color::new(211, 211, 211);

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
            gap: 4,
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
