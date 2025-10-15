use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::Path,
    sync::{Arc, LazyLock},
};

use serde::Deserialize;
use tracing::info;

use crate::engines::Engine;

impl Default for Config {
    fn default() -> Self {
        Config {
            bind: "0.0.0.0:28019".parse().unwrap(),
            api: false,
            ui: UiConfig {
                show_engine_list_separator: false,
                show_version_info: false,
                site_name: "metasearch".to_string(),
                show_settings_link: true,
                stylesheet_url: "".to_string(),
                stylesheet_str: "".to_string(),
                favicon_url: "".to_string(),
                show_autocomplete: true,
            },
            image_search: ImageSearchConfig {
                enabled: false,
                show_engines: true,
                proxy: ImageProxyConfig {
                    enabled: true,
                    max_download_size: 10_000_000,
                },
            },
            engines: Arc::new(EnginesConfig::default()),
            urls: UrlsConfig {
                replace: vec![(
                    HostAndPath::new("minecraft.fandom.com/wiki/"),
                    HostAndPath::new("minecraft.wiki/w/"),
                )],
                weight: vec![],
            },
        }
    }
}

impl Default for EnginesConfig {
    fn default() -> Self {
        use toml::value::Value;

        let mut map = HashMap::new();
        // engines are enabled by default, so engines that aren't listed here are
        // enabled

        // main search engines
        map.insert(Engine::Google, EngineConfig::new().with_weight(1.05));
        map.insert(Engine::Bing, EngineConfig::new().with_weight(1.0));
        map.insert(Engine::Brave, EngineConfig::new().with_weight(1.25));
        map.insert(
            Engine::Marginalia,
            EngineConfig::new().with_weight(0.15).with_extra(
                vec![(
                    "args".to_string(),
                    Value::Table(
                        vec![
                            ("profile".to_string(), Value::String("corpo".to_string())),
                            ("js".to_string(), Value::String("default".to_string())),
                            ("adtech".to_string(), Value::String("default".to_string())),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                )]
                .into_iter()
                .collect(),
            ),
        );

        // additional search engines
        map.insert(
            Engine::GoogleScholar,
            EngineConfig::new().with_weight(0.50).disabled(),
        );
        map.insert(
            Engine::RightDao,
            EngineConfig::new().with_weight(0.10).disabled(),
        );
        map.insert(
            Engine::Stract,
            EngineConfig::new().with_weight(0.15).disabled(),
        );
        map.insert(
            Engine::Yep,
            EngineConfig::new().with_weight(0.10).disabled(),
        );

        // calculators (give them a high weight so they're always the first thing in
        // autocomplete)
        map.insert(Engine::Numbat, EngineConfig::new().with_weight(10.0));
        map.insert(
            Engine::Fend,
            EngineConfig::new().with_weight(10.0).disabled(),
        );

        // other engines
        map.insert(
            Engine::Mdn,
            EngineConfig::new().with_extra(
                vec![("max_sections".to_string(), Value::Integer(1))]
                    .into_iter()
                    .collect(),
            ),
        );

        Self { map }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            weight: 1.0,
            extra: Default::default(),
        }
    }
}
static DEFAULT_ENGINE_CONFIG_REF: LazyLock<EngineConfig> = LazyLock::new(EngineConfig::default);
impl EngineConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_weight(self, weight: f64) -> Self {
        Self { weight, ..self }
    }
    pub fn disabled(self) -> Self {
        Self {
            enabled: false,
            ..self
        }
    }
    pub fn with_extra(self, extra: toml::Table) -> Self {
        Self { extra, ..self }
    }
}

//

#[derive(Debug, Clone)]
pub struct Config {
    pub bind: SocketAddr,
    /// Whether the JSON API should be accessible.
    pub api: bool,
    pub ui: UiConfig,
    pub image_search: ImageSearchConfig,
    // wrapped in an arc to make Config cheaper to clone
    pub engines: Arc<EnginesConfig>,
    pub urls: UrlsConfig,
}

#[derive(Deserialize, Debug)]
pub struct PartialConfig {
    pub bind: Option<SocketAddr>,
    pub api: Option<bool>,
    pub ui: Option<PartialUiConfig>,
    pub image_search: Option<PartialImageSearchConfig>,
    pub engines: Option<PartialEnginesConfig>,
    pub urls: Option<PartialUrlsConfig>,
}

impl Config {
    pub fn overlay(&mut self, partial: PartialConfig) {
        self.bind = partial.bind.unwrap_or(self.bind);
        self.api = partial.api.unwrap_or(self.api);
        self.ui.overlay(partial.ui.unwrap_or_default());
        self.image_search
            .overlay(partial.image_search.unwrap_or_default());
        if let Some(partial_engines) = partial.engines {
            let mut engines = self.engines.as_ref().clone();
            engines.overlay(partial_engines);
            self.engines = Arc::new(engines);
        }
        self.urls.overlay(partial.urls.unwrap_or_default());
    }
}

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub show_engine_list_separator: bool,
    pub show_version_info: bool,
    /// Settings are always accessible anyways, this just controls whether the
    /// link to them in the index page is visible.
    pub show_settings_link: bool,
    pub site_name: String,
    pub show_autocomplete: bool,
    pub stylesheet_url: String,
    pub stylesheet_str: String,
    pub favicon_url: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialUiConfig {
    pub show_engine_list_separator: Option<bool>,
    pub show_version_info: Option<bool>,
    pub show_settings_link: Option<bool>,
    pub show_autocomplete: Option<bool>,

    pub site_name: Option<String>,
    pub stylesheet_url: Option<String>,
    pub stylesheet_str: Option<String>,
    pub favicon_url: Option<String>,
}

impl UiConfig {
    pub fn overlay(&mut self, partial: PartialUiConfig) {
        self.show_engine_list_separator = partial
            .show_engine_list_separator
            .unwrap_or(self.show_engine_list_separator);
        self.show_version_info = partial.show_version_info.unwrap_or(self.show_version_info);
        self.show_settings_link = partial
            .show_settings_link
            .unwrap_or(self.show_settings_link);
        self.show_autocomplete = partial.show_autocomplete.unwrap_or(self.show_autocomplete);
        self.site_name = partial.site_name.unwrap_or(self.site_name.clone());
        self.stylesheet_url = partial
            .stylesheet_url
            .unwrap_or(self.stylesheet_url.clone());
        self.stylesheet_str = partial
            .stylesheet_str
            .unwrap_or(self.stylesheet_str.clone());
        self.favicon_url = partial.favicon_url.unwrap_or(self.favicon_url.clone());
    }
}

#[derive(Debug, Clone)]
pub struct ImageSearchConfig {
    pub enabled: bool,
    pub show_engines: bool,
    pub proxy: ImageProxyConfig,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialImageSearchConfig {
    pub enabled: Option<bool>,
    pub show_engines: Option<bool>,
    pub proxy: Option<PartialImageProxyConfig>,
}

impl ImageSearchConfig {
    pub fn overlay(&mut self, partial: PartialImageSearchConfig) {
        self.enabled = partial.enabled.unwrap_or(self.enabled);
        self.show_engines = partial.show_engines.unwrap_or(self.show_engines);
        self.proxy.overlay(partial.proxy.unwrap_or_default());
    }
}

#[derive(Debug, Clone)]
pub struct ImageProxyConfig {
    /// Whether we should proxy remote images through our server. This is mostly
    /// a privacy feature.
    pub enabled: bool,
    /// The maximum size of an image that can be proxied. This is in bytes.
    pub max_download_size: u64,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialImageProxyConfig {
    pub enabled: Option<bool>,
    pub max_download_size: Option<u64>,
}

impl ImageProxyConfig {
    pub fn overlay(&mut self, partial: PartialImageProxyConfig) {
        self.enabled = partial.enabled.unwrap_or(self.enabled);
        self.max_download_size = partial.max_download_size.unwrap_or(self.max_download_size);
    }
}

#[derive(Debug, Clone)]
pub struct EnginesConfig {
    pub map: HashMap<Engine, EngineConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct PartialEnginesConfig {
    #[serde(flatten)]
    pub map: HashMap<Engine, PartialDefaultableEngineConfig>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum PartialDefaultableEngineConfig {
    Boolean(bool),
    Full(PartialEngineConfig),
}

impl EnginesConfig {
    pub fn overlay(&mut self, partial: PartialEnginesConfig) {
        for (key, value) in partial.map {
            let full = match value {
                PartialDefaultableEngineConfig::Boolean(enabled) => PartialEngineConfig {
                    enabled: Some(enabled),
                    ..Default::default()
                },
                PartialDefaultableEngineConfig::Full(full) => full,
            };
            if let Some(existing) = self.map.get_mut(&key) {
                existing.overlay(full);
            } else {
                let mut new = EngineConfig::default();
                new.overlay(full);
                self.map.insert(key, new);
            }
        }
    }

    pub fn get(&self, engine: Engine) -> &EngineConfig {
        self.map.get(&engine).unwrap_or(&DEFAULT_ENGINE_CONFIG_REF)
    }
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub enabled: bool,
    /// The priority of this engine relative to the other engines.
    pub weight: f64,
    /// Per-engine configs. These are parsed at request time.
    pub extra: toml::Table,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct PartialEngineConfig {
    pub enabled: Option<bool>,
    pub weight: Option<f64>,
    #[serde(flatten)]
    pub extra: toml::Table,
}

impl EngineConfig {
    pub fn overlay(&mut self, partial: PartialEngineConfig) {
        self.enabled = partial.enabled.unwrap_or(self.enabled);
        self.weight = partial.weight.unwrap_or(self.weight);
        self.extra.extend(partial.extra);
    }
}

impl Config {
    pub fn read_or_create(config_path: &Path) -> eyre::Result<Self> {
        let mut config = Config::default();

        if !config_path.exists() {
            info!("No config found, creating one at {config_path:?}");
            let default_config_str = include_str!("../config-default.toml");
            if let Some(parent_path) = config_path.parent() {
                let _ = fs::create_dir_all(parent_path);
            }
            fs::write(config_path, default_config_str)?;
        }

        let given_config = toml::from_str::<PartialConfig>(&fs::read_to_string(config_path)?)?;
        config.overlay(given_config);
        Ok(config)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HostAndPath {
    pub host: String,
    pub path: String,
}
impl HostAndPath {
    pub fn new(s: &str) -> Self {
        let (host, path) = s.split_once('/').unwrap_or((s, ""));
        Self {
            host: host.to_owned(),
            path: path.to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlsConfig {
    pub replace: Vec<(HostAndPath, HostAndPath)>,
    pub weight: Vec<(HostAndPath, f64)>,
}
#[derive(Deserialize, Debug, Default)]
pub struct PartialUrlsConfig {
    #[serde(default)]
    pub replace: HashMap<String, String>,
    #[serde(default)]
    pub weight: HashMap<String, f64>,
}
impl UrlsConfig {
    pub fn overlay(&mut self, partial: PartialUrlsConfig) {
        for (from, to) in partial.replace {
            let from = HostAndPath::new(&from);
            if to.is_empty() {
                // setting the value to an empty string removes it
                let index = self.replace.iter().position(|(u, _)| u == &from);
                // swap_remove is fine because the order of this vec doesn't matter
                self.replace.swap_remove(index.unwrap());
            } else {
                let to = HostAndPath::new(&to);
                self.replace.push((from, to));
            }
        }

        for (url, weight) in partial.weight {
            let url = HostAndPath::new(&url);
            self.weight.push((url, weight));
        }

        // sort by length so that more specific checls are done first
        self.weight.sort_by(|(a, _), (b, _)| {
            let a_len = a.path.len() + a.host.len();
            let b_len = b.path.len() + b.host.len();
            b_len.cmp(&a_len)
        });
    }
}
