use crate::domain::{BenchmarkNumRuns, QueryResults};
use crate::repository::{DbClient, QueryExecutor};
use anyhow::Context;
use colored::Colorize;
use std::time::Instant;

pub async fn execute_query(db_client: &DbClient, query: &str) -> anyhow::Result<QueryResults> {
    let results = db_client.execute_query(query).await?;

    Ok(results)
}

pub async fn benchmark_query(
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
