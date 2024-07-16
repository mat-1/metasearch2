use std::{
    env,
    path::{Path, PathBuf},
};

use config::Config;
use tracing::error;

pub mod config;
pub mod engines;
pub mod parse;
pub mod urls;
pub mod web;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    if env::args().any(|arg| arg == "--help" || arg == "-h" || arg == "help" || arg == "h") {
        println!("Usage: metasearch [config_path]");
        return;
    }

    let config_path = config_path();
    let config = match Config::read_or_create(&config_path) {
        Ok(config) => config,
        Err(err) => {
            error!("Couldn't parse config:\n{err}");
            return;
        }
    };
    web::run(config).await;
}

fn config_path() -> PathBuf {
    if let Some(config_path) = env::args().nth(1) {
        return PathBuf::from(config_path);
    }

    let app_name = env!("CARGO_PKG_NAME");

    let mut default_config_dir = None;

    // $XDG_CONFIG_HOME/metasearch/config.toml
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config_home)
            .join(app_name)
            .join("config.toml");
        if path.is_file() {
            return path;
        }
        if default_config_dir.is_none() {
            default_config_dir = Some(path);
        }
    }

    // $HOME/.config/metasearch/config.toml
    if let Ok(home) = env::var("HOME") {
        let path = PathBuf::from(home)
            .join(".config")
            .join(app_name)
            .join("config.toml");
        if path.is_file() {
            return path;
        }
        if default_config_dir.is_none() {
            default_config_dir = Some(path);
        }
    }

    // ./config.toml
    let path = Path::new("config.toml");
    if path.exists() {
        return path.to_path_buf();
    }
    default_config_dir.unwrap_or(PathBuf::from("config.toml"))
}
