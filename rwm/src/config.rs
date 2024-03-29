use std::{collections::HashMap, convert::TryFrom, fs::File, path::PathBuf};

use {
    anyhow::{bail, Context, Result},
    serde::{Deserialize, Serialize},
};

use crate::{color::Color, layouts::LayoutType, mod_mask::XModMask, spawn_rule::SpawnRule};
use common::TagId;

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
    /// This is used only for printing and reading to and from a config file
    /// It gets broken to `class_rules` and `name_rules`, these are actually used by the wm
    rules: Vec<SpawnRule>,
    #[serde(skip)]
    pub(crate) class_rules: HashMap<String, Vec<TagId>>,
    #[serde(skip)]
    pub(crate) name_rules: HashMap<String, Vec<TagId>>,
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
            rules: vec![],
            class_rules: HashMap::new(),
            name_rules: HashMap::new(),
            path: None,
        }
    }
}

impl Config {
    /// Split SpawnRules to two HashMaps, one with class rules and one with name rules
    fn extract_rules(
        rules: &[SpawnRule],
    ) -> (HashMap<String, Vec<TagId>>, HashMap<String, Vec<TagId>>) {
        let mut class_rules = HashMap::new();
        let mut name_rules = HashMap::new();
        for rule in rules.iter() {
            match rule {
                SpawnRule::ClassName(name, tag_ids) => {
                    class_rules.insert(name.to_owned(), tag_ids.to_owned())
                }
                SpawnRule::WmName(name, tag_ids) => {
                    name_rules.insert(name.to_owned(), tag_ids.to_owned())
                }
            };
        }

        (class_rules, name_rules)
    }

    pub(crate) fn from_file(path: PathBuf) -> Result<Self> {
        let conf_file =
            File::open(&path).context(format!("Failed to open `{}`", path.display()))?;
        let mut config: Self = ron::de::from_reader(conf_file)
            .context(format!("Failed to parse `{}`", path.display()))?;

        log::info!("Loaded config from file {}", path.display());
        config.path = Some(path);

        let (class_rules, name_rules) = Self::extract_rules(&config.rules);
        config.class_rules = class_rules;
        config.name_rules = name_rules;

        Ok(config)
    }

    pub(crate) fn load(&mut self, path: Option<PathBuf>) -> Result<()> {
        if path.is_none() && self.path.is_none() {
            bail!("No configuration file specified");
        }
        let path = path.unwrap_or_else(|| self.path.clone().unwrap());

        log::trace!("Replacing config with {}", path.display());

        let conf_file =
            File::open(&path).context(format!("Failed to open `{}`", path.display()))?;
        let mut new_config: Self = ron::de::from_reader(conf_file)
            .context(format!("Failed to parse `{}`", path.display()))?;
        new_config.path = Some(path);

        let _ = std::mem::replace(self, new_config);

        let (class_rules, name_rules) = Self::extract_rules(&self.rules);
        self.class_rules = class_rules;
        self.name_rules = name_rules;

        log::info!("Loaded config from file");
        Ok(())
    }
}
