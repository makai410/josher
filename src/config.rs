use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    /// Projection from the full mono repo history.
    pub mono_filter: String,

    /// Projection from the full subtree repo history.
    pub subtree_filter: String,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .context(format!("cannot load config from {}", path.display()))?;
        let config: Self = toml::from_str(&text).context("cannot load config as toml")?;
        config.validate()?;
        Ok(config)
    }

    pub fn write(&self, path: &Path) -> anyhow::Result<()> {
        self.validate()?;
        let text = toml::to_string_pretty(self)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .context(format!("cannot create config at {}", path.display()))?;
        file.write_all(text.as_bytes())
            .context(format!("cannot write config to {}", path.display()))
    }

    fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.mono_filter.trim().is_empty(),
            "`mono-filter` cannot be empty"
        );
        Ok(())
    }
}
