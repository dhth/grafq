mod cli;
mod cmds;
mod config;
mod domain;
mod logging;
mod repository;
mod service;
mod utils;
mod view;

use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_neptunedata::config::ProvideCredentials;
use chrono::Utc;
use clap::Parser;
use cli::Args;
use domain::QueryResults;
use etcetera::BaseStrategy;
use repository::{DbClient, Neo4jClient, Neo4jConfig, NeptuneClient};
use std::io::Read;
use view::Console;
use view::{ConsoleConfig, get_results};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let xdg = etcetera::choose_base_strategy()?;
    logging::setup(&xdg)?;
    let args = Args::parse();

    if args.debug {
        print!("DEBUG INFO\n{args}");
        return Ok(());
    }

    match args.command {
        cli::GraphQCommand::Console {
            page_results,
            write_results,
            results_directory,
            results_format,
        } => {
            let db_client = get_db_client().await?;
            let history_file_path = xdg.data_dir().join("gcue").join("history.txt");

            if let Some(parent) = history_file_path.parent() {
                tokio::fs::create_dir_all(parent).await.with_context(|| {
                    format!("couldn't create directory for gcue's history: {:?}", parent,)
                })?;
            }

            let console_config = ConsoleConfig {
                page_results,
                write_results,
                results_directory,
                results_format,
            };

            let pager = if page_results {
                Some(utils::get_pager()?)
            } else {
                None
            };

            let mut console = Console::new(db_client, history_file_path, console_config, pager);
            console.run_loop().await?;
        }
        cli::GraphQCommand::Query {
            query,
            page_results,
            benchmark,
            bench_num_runs,
            bench_num_warmup_runs,
            print_query,
            write_results,
            results_directory,
            results_format,
        } => {
            if benchmark && write_results {
                anyhow::bail!("cannot benchmark and write results at the same time");
            }

            let pager = if page_results {
                Some(utils::get_pager()?)
            } else {
                None
            };

            let db_client = get_db_client().await?;

            let query = if query.as_str() == "-" {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .context("couldn't read query from stdin")?;
                buffer.trim().to_string()
            } else {
                query
            };

            if print_query {
                println!(
                    r#"---
{query}
---
"#
                );
            }

            if benchmark {
                cmds::benchmark_query(&db_client, &query, bench_num_runs, bench_num_warmup_runs)
                    .await?;
            } else {
                let results = cmds::execute_query(&db_client, &query).await?;
                let results = match results {
                    QueryResults::Empty => {
                        println!("No results");
                        return Ok(());
                    }
                    QueryResults::NonEmpty(res) => res,
                };

                if write_results {
                    let results_file_path = crate::service::write_results(
                        &results,
                        &results_directory,
                        &results_format,
                        Utc::now(),
                    )
                    .context("couldn't write results")?;
                    println!("Wrote results to {}", results_file_path.to_string_lossy());

                    if let Some(pager) = pager {
                        service::page_results(&results_file_path, &pager)?;
                    }
                } else if let Some(pager) = pager {
                    let temp_results_directory = tempfile::tempdir()
                        .context("couldn't create temporary directory for paging results")?;
                    let results_file_path = crate::service::write_results(
                        &results,
                        &temp_results_directory,
                        &results_format,
                        Utc::now(),
                    )
                    .context("couldn't write results to temporary location")?;

                    service::page_results(&results_file_path, &pager)?;
                } else {
                    let results_str = get_results(&results);
                    println!("{}", results_str);
                }
            }
        }
    }

    Ok(())
}

async fn get_db_client() -> anyhow::Result<DbClient> {
    let db_uri = utils::get_mandatory_env_var("DB_URI")?;

    let db_client = match db_uri.split_once("://") {
        Some(("http", _)) | Some(("https", _)) => {
            let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            if let Some(provider) = sdk_config.credentials_provider() {
                provider
                    .provide_credentials()
                    .await
                    .context("couldn't fetch AWS credentials")?;
            }

            let neptune_client = NeptuneClient::new(&sdk_config, &db_uri);
            DbClient::Neptune(neptune_client)
        }
        Some(("bolt", _)) => {
            let user = utils::get_mandatory_env_var("NEO4J_USER")?;
            let password = utils::get_mandatory_env_var("NEO4J_PASSWORD")?;
            let database_name = utils::get_mandatory_env_var("NEO4J_DB")?;

            let config = Neo4jConfig {
                db_uri,
                user,
                password,
                database_name,
            };

            let neo4j_client = Neo4jClient::new(&config).await?;
            DbClient::Neo4j(neo4j_client)
        }
        Some((_, _)) => {
            anyhow::bail!("db uri must have one of the following protocols: [http, https, bolt]")
        }
        None => anyhow::bail!(
            r#"db uri must be a valid uri, eg. "bolt://127.0.0.1:7687", or "https://abc.xyz.us-east-1.neptune.amazonaws.com:8182""#
        ),
    };

    Ok(db_client)
}
