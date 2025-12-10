use crate::domain::{BenchmarkNumRuns, OutputFormat, QueryResults};
use crate::repository::{DbClient, DbClientError, QueryExecutor, get_db_client};
use crate::utils::get_pager;
use crate::view::get_results;
use anyhow::Context;
use chrono::Utc;
use colored::Colorize;
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;

pub enum QueryBehaviour {
    Benchmark {
        num_runs: BenchmarkNumRuns,
        warmup_runs: u16,
    },
    Normal {
        page_results: bool,
        write_results: bool,
        results_directory: PathBuf,
        results_format: OutputFormat,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum QueryCmdError {
    #[error("couldn't build db client: {0}")]
    CouldntBuildDbClient(#[from] DbClientError),
    #[error(transparent)]
    Uncategorised(#[from] anyhow::Error),
}

pub async fn handle_query_cmd(
    query: String,
    behaviour: QueryBehaviour,
    print_query: bool,
) -> Result<(), QueryCmdError> {
    let db_client = get_db_client().await?;

    let query = if query == "-" {
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
    match behaviour {
        QueryBehaviour::Benchmark {
            num_runs,
            warmup_runs,
        } => {
            benchmark_query(&db_client, &query, num_runs, warmup_runs).await?;
        }

        QueryBehaviour::Normal {
            page_results,
            write_results,
            results_directory,
            results_format,
        } => {
            let pager = if page_results {
                Some(get_pager()?)
            } else {
                None
            };

            let results = db_client.execute_query(&query).await?;
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
                    crate::service::page_results(&results_file_path, &pager)?;
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

                crate::service::page_results(&results_file_path, &pager)?;
            } else {
                let results_str = get_results(&results);
                println!("{}", results_str);
            }
        }
    }

    Ok(())
}

async fn benchmark_query(
    db_client: &DbClient,
    query: &str,
    num_runs: BenchmarkNumRuns,
    num_warmup_runs: u16,
) -> anyhow::Result<()> {
    if num_warmup_runs > 0 {
        println!(
            "{}",
            format!("Warming up ({num_warmup_runs} runs) ...")
                .yellow()
                .bold()
        );
    }
    for i in 0..num_warmup_runs {
        let start = Instant::now();
        db_client
            .execute_query(query)
            .await
            .with_context(|| format!("couldn't get results for warmup run #{}", i + 1))?;
        let elapsed = start.elapsed().as_millis();
        println!("run {:03}:      {}", i + 1, format!("{}ms", elapsed).cyan(),);
    }

    if num_warmup_runs > 0 {
        println!();
    }

    println!(
        "{}",
        format!("Benchmarking ({} runs) ...", num_runs.value())
            .yellow()
            .bold()
    );

    let mut times = vec![];
    for i in 0..num_runs.value() {
        let start = Instant::now();
        db_client
            .execute_query(query)
            .await
            .with_context(|| format!("couldn't execute query for benchmark run #{}", i + 1))?;
        let elapsed = start.elapsed().as_millis();
        println!("run {:03}:      {}", i + 1, format!("{}ms", elapsed).cyan(),);
        times.push(elapsed);
    }

    if let (Some(min), Some(max)) = (times.iter().min(), times.iter().max()) {
        let mean = times.iter().sum::<u128>() / times.len() as u128;
        print!(
            "
{}
min:          {}
max:          {}
mean:         {}
",
            "Statistics:".yellow().bold(),
            format!("{}ms", min).cyan(),
            format!("{}ms", max).cyan(),
            format!("{}ms", mean).cyan(),
        );
    }

    Ok(())
}
