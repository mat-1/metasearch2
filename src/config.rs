use std::{collections::HashMap, fs, net::SocketAddr, path::Path};

use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::engines::Engine;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    pub engine_list_separator: bool,
    pub engines: EnginesConfig,
}

impl Config {
    pub fn read_or_create() -> eyre::Result<Self> {
        let default_config_str = include_str!("../default-config.toml");
        let mut config: Config = toml::from_str(default_config_str)?;

        let config_path = Path::new("config.toml");
        if config_path.exists() {
            let given_config = toml::from_str::<Config>(&fs::read_to_string(config_path)?)?;
            config.update(given_config);
            Ok(config)
        } else {
            println!("No config found, creating one at {config_path:?}");
            fs::write(config_path, default_config_str)?;
            Ok(config)
        }
    }

    // Update the current config with the given config. This is used to make it so
    // the default-config.toml is always used as a fallback if the user decides to
    // use the default for something.
    pub fn update(&mut self, other: Self) {
        self.bind = other.bind;
        self.engine_list_separator |= other.engine_list_separator;
        for (key, value) in other.engines.map {
            if let Some(existing) = self.engines.map.get_mut(&key) {
                existing.update(value);
            } else {
                self.engines.map.insert(key, value);
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct EnginesConfig {
    #[serde(flatten)]
    pub map: HashMap<Engine, DefaultableEngineConfig>,
}

static DEFAULT_ENABLED_FULL_ENGINE_CONFIG: Lazy<FullEngineConfig> =
    Lazy::new(FullEngineConfig::default);
static DEFAULT_DISABLED_FULL_ENGINE_CONFIG: Lazy<FullEngineConfig> =
    Lazy::new(|| FullEngineConfig {
        enabled: false,
        ..Default::default()
    });

impl EnginesConfig {
    pub fn get(&self, engine: Engine) -> &FullEngineConfig {
        match self.map.get(&engine) {
            Some(engine_config) => match engine_config {
                DefaultableEngineConfig::Boolean(enabled) => {
                    if *enabled {
                        &DEFAULT_ENABLED_FULL_ENGINE_CONFIG
                    } else {
                        &DEFAULT_DISABLED_FULL_ENGINE_CONFIG
                    }
                }
                DefaultableEngineConfig::Full(full) => full,
            },
            None => &DEFAULT_ENABLED_FULL_ENGINE_CONFIG,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum DefaultableEngineConfig {
    Boolean(bool),
    Full(FullEngineConfig),
}

impl DefaultableEngineConfig {
    pub fn update(&mut self, other: Self) {
        match (self, other) {
            (Self::Boolean(existing), Self::Boolean(other)) => *existing = other,
            (Self::Full(existing), Self::Full(other)) => existing.update(other),
            _ => (),
        }
    }
}

impl Default for DefaultableEngineConfig {
    fn default() -> Self {
        Self::Boolean(true)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FullEngineConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// The priority of this engine relative to the other engines. The default
    /// is 1, and a value of 0 is treated as the default.
    #[serde(default)]
    pub weight: f64,
    /// Per-engine configs. These are parsed at request time.
    #[serde(flatten)]
    #[serde(default)]
    pub extra: toml::Table,
}

// serde expects a function as the default, this just exists so "enabled" is
// always true by default
fn default_true() -> bool {
    true
}

impl From<DefaultableEngineConfig> for FullEngineConfig {
    fn from(config: DefaultableEngineConfig) -> Self {
        match config {
            DefaultableEngineConfig::Boolean(enabled) => Self {
                enabled,
                ..Default::default()
            },
            DefaultableEngineConfig::Full(full) => full,
        }
    }
}

impl Default for FullEngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            weight: 1.0,
            extra: Default::default(),
        }
    }
}

impl FullEngineConfig {
    pub fn update(&mut self, other: Self) {
        self.enabled = other.enabled;
        if other.weight != 0. {
            self.weight = other.weight;
        }
        self.extra = other.extra;
    }
}
