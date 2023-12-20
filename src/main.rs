#![feature(lazy_cell)]

pub mod engines;
pub mod normalize;
pub mod parse;
pub mod web;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    web::run().await;
}
