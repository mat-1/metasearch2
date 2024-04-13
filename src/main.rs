use config::Config;

pub mod config;
pub mod engines;
pub mod normalize;
pub mod parse;
pub mod web;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = match Config::read_or_create() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Couldn't parse config:\n{err}");
            return;
        }
    };
    web::run(config).await;
}
