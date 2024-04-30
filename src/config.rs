use std::{collections::HashMap, fs, net::SocketAddr, path::Path};

use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::info;

use crate::engines::Engine;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub bind: SocketAddr,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub image_search: ImageSearchConfig,
    #[serde(default)]
    pub engines: EnginesConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct UiConfig {
    #[serde(default)]
    pub show_engine_list_separator: Option<bool>,
    #[serde(default)]
    pub show_version_info: Option<bool>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ImageSearchConfig {
    pub enabled: Option<bool>,
    #[serde(default)]
    pub proxy: ImageProxyConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct ImageProxyConfig {
    /// Whether we should proxy remote images through our server. This is mostly
    /// a privacy feature.
    pub enabled: Option<bool>,
    /// The maximum size of an image that can be proxied. This is in bytes.
    pub max_download_size: Option<u64>,
}

#[derive(Deserialize, Debug, Default)]
pub struct EnginesConfig {
    #[serde(flatten)]
    pub map: HashMap<Engine, DefaultableEngineConfig>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum DefaultableEngineConfig {
    Boolean(bool),
    Full(FullEngineConfig),
}

#[derive(Deserialize, Clone, Debug)]
pub struct FullEngineConfig {
    #[serde(default = "fn_true")]
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

impl Config {
    pub fn read_or_create(config_path: &Path) -> eyre::Result<Self> {
        let base_config_str = include_str!("../config-base.toml");
        let mut config: Config = toml::from_str(base_config_str)?;

        if !config_path.exists() {
            info!("No config found, creating one at {config_path:?}");
            let default_config_str = include_str!("../config-default.toml");
            fs::write(config_path, default_config_str)?;
        }

        let given_config = toml::from_str::<Config>(&fs::read_to_string(config_path)?)?;
        config.update(given_config);
        Ok(config)
    }

    // Update the current config with the given config. This is used to make it so
    // the config-base.toml is always used as a fallback if the user decides to
    // use the default for something.
    pub fn update(&mut self, new: Config) {
        self.bind = new.bind;
        self.ui.update(new.ui);
        self.image_search.update(new.image_search);
        self.engines.update(new.engines);
    }
}

impl UiConfig {
    pub fn update(&mut self, new: UiConfig) {
        self.show_engine_list_separator = new
            .show_engine_list_separator
            .or(self.show_engine_list_separator);
        assert_ne!(self.show_engine_list_separator, None);
        self.show_version_info = new.show_version_info.or(self.show_version_info);
        assert_ne!(self.show_version_info, None);
    }
}

impl ImageSearchConfig {
    pub fn update(&mut self, new: ImageSearchConfig) {
        self.enabled = new.enabled.or(self.enabled);
        assert_ne!(self.enabled, None);
        self.proxy.update(new.proxy);
    }
}

impl ImageProxyConfig {
    pub fn update(&mut self, new: ImageProxyConfig) {
        self.enabled = new.enabled.or(self.enabled);
        assert_ne!(self.enabled, None);
        self.max_download_size = new.max_download_size.or(self.max_download_size);
        assert_ne!(self.max_download_size, None);
    }
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

    pub fn update(&mut self, new: Self) {
        for (key, new) in new.map {
            if let Some(existing) = self.map.get_mut(&key) {
                existing.update(new);
            } else {
                self.map.insert(key, new);
            }
        }
    }
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

// serde expects a function as the default, this just exists so "enabled" is
// always true by default
fn fn_true() -> bool {
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
