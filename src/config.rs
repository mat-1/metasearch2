use std::{collections::HashMap, fs, net::SocketAddr, path::Path};

use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::info;

use crate::engines::Engine;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    #[serde(default)]
    pub engine_list_separator: Option<bool>,
    #[serde(default)]
    pub version_info: Option<bool>,
    pub engines: EnginesConfig,
}

impl Config {
    pub fn read_or_create() -> eyre::Result<Self> {
        let default_config_str = include_str!("../default-config.toml");
        let mut config: Config = toml::from_str(default_config_str)?;

        let config_path = std::env::args().next().unwrap_or("config.toml".into());
        let config_path = Path::new(&config_path);
        if config_path.exists() {
            let given_config = toml::from_str::<Config>(&fs::read_to_string(config_path)?)?;
            config.update(given_config);
            Ok(config)
        } else {
            info!("No config found, creating one at {config_path:?}");
            fs::write(config_path, default_config_str)?;
            Ok(config)
        }
    }

    // Update the current config with the given config. This is used to make it so
    // the default-config.toml is always used as a fallback if the user decides to
    // use the default for something.
    pub fn update(&mut self, new: Config) {
        self.bind = new.bind;
        self.engine_list_separator = new.engine_list_separator.or(self.engine_list_separator);
        assert_ne!(self.engine_list_separator, None);
        self.version_info = new.version_info.or(self.version_info);
        assert_ne!(self.version_info, None);
        for (key, new) in new.engines.map {
            if let Some(existing) = self.engines.map.get_mut(&key) {
                existing.update(new);
            } else {
                self.engines.map.insert(key, new);
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
    pub fn update(&mut self, new: Self) {
        let mut self_full = FullEngineConfig::from(self.clone());
        let other_full = FullEngineConfig::from(new);
        self_full.update(other_full);
        *self = DefaultableEngineConfig::Full(self_full);
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
    pub fn update(&mut self, new: Self) {
        self.enabled = new.enabled;
        if new.weight != 0. {
            self.weight = new.weight;
        }
        self.extra = new.extra;
    }
}
