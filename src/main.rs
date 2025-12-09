mod cli;
mod cmds;
mod config;
mod domain;
mod logging;
mod repository;
mod run;
mod service;
mod utils;
mod view;

#[tokio::main]
async fn main() {
    if let Err(e) = run::run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
