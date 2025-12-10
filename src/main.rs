mod app;
mod cli;
mod cmds;
mod config;
mod domain;
mod error;
mod logging;
mod repository;
mod service;
mod utils;
mod view;

#[tokio::main]
async fn main() {
    if let Err(e) = app::run().await {
        eprintln!("Error: {:#}", e);

        if let Some(follow_up) = e.follow_up() {
            eprintln!("\n{follow_up}");
        }

        if e.is_unexpected() {
            eprint!(
                "
---
This error is unexpected. 
Let @dhth know about this via https://github.com/dhth/grf/issues.
"
            );
        }

        std::process::exit(1);
    }
}
