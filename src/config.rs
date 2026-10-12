use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub mono: RepoConfig,
    pub subtree: RepoConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RepoConfig {
    pub org: String,
    pub repo: String,
    // FIXME: we need a smart url parser here
    pub url: String,
    #[serde(default)]
    pub filter: String,
}

impl Config {
    pub fn template() -> Self {
        Self {
            mono: RepoConfig {
                org: "<organization name>".to_string(),
                repo: "<mono repository name>".to_string(),
                url: "url to the repository, both http & ssh are okay".to_string(),
                filter: "<mono filter>".to_string(),
            },
            subtree: RepoConfig {
                org: "<organization name>".to_string(),
                repo: "<subtree repository name>".to_string(),
                url: "url to the repository, both http & ssh are okay".to_string(),
                filter:
                    "<subtree filter> (optional, if you want to mirror the entire subtree repo)"
                        .to_string(),
            },
        }
    }
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
            !self.mono.filter.trim().is_empty(),
            "`mono.filter` cannot be empty"
        );
        Ok(())
    }
}
