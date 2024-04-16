use config::Config;
use tracing::error;

pub mod config;
pub mod engines;
pub mod normalize;
pub mod parse;
pub mod web;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = match Config::read_or_create() {
        Ok(config) => config,
        Err(err) => {
            error!("Couldn't parse config:\n{err}");
            return;
        }
    };
    web::run(config).await;
}
