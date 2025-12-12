use crate::config::DEFAULT_RESULTS_DIR;
use crate::domain::{BenchmarkNumRuns, ResultsFormat};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// grafq lets you query Neo4j/AWS Neptune databases via an interactive console
#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: GraphQCommand,
    /// Output debug information without doing anything
    #[arg(long = "debug", global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum GraphQCommand {
    /// Open grafq's console
    #[command()]
    Console {
        /// Display results via a pager ("less", by default, can be overridden by $GRAFQ_PAGER)
        #[arg(short = 'p', long = "page-results")]
        page_results: bool,
        /// Write results to filesystem
        #[arg(short = 'w', long = "write-results")]
        write_results: bool,
        /// Directory to write results in
        #[arg(
            short = 'd',
            long = "results-dir",
            value_name = "DIRECTORY",
            default_value = DEFAULT_RESULTS_DIR,
        )]
        results_directory: PathBuf,
        /// Format to write results in
        #[arg(
            short = 'f',
            long = "results-format",
            value_name = "FORMAT",
            default_value = "json"
        )]
        results_format: ResultsFormat,
    },
    /// Execute a one-off query
    #[command()]
    Query {
        /// Display results via a pager ("less", by default, can be overridden by $GRAFQ_PAGER)
        #[arg(short = 'p', long = "page-results")]
        page_results: bool,
        /// Cypher query to execute
        #[arg()]
        query: String,
        /// Whether to benchmark the query
        #[arg(short = 'b', long = "bench")]
        benchmark: bool,
        /// Number of benchmark runs
        #[arg(
            short = 'n',
            long = "bench-num-runs",
            default_value = "5",
            value_name = "NUMBER"
        )]
        bench_num_runs: BenchmarkNumRuns,
        /// Number of benchmark warmup runs
        #[arg(
            short = 'W',
            long = "bench-num-warmup-runs",
            default_value_t = 3,
            value_name = "NUMBER"
        )]
        bench_num_warmup_runs: u16,
        /// Print query
        #[arg(short = 'P', long = "print-query")]
        print_query: bool,
        /// Write results to filesystem
        #[arg(short = 'w', long = "write-results")]
        write_results: bool,
        /// Directory to write results in
        #[arg(
            short = 'd',
            long = "results-dir",
            value_name = "DIRECTORY",
            default_value = DEFAULT_RESULTS_DIR,
        )]
        results_directory: PathBuf,
        /// Format to write results in
        #[arg(
            short = 'f',
            long = "results-format",
            value_name = "FORMAT",
            default_value = "json"
        )]
        results_format: ResultsFormat,
    },
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match &self.command {
            GraphQCommand::Console {
                page_results,
                write_results,
                results_directory,
                results_format,
            } => format!(
                "
command:                    console
display results via pager:  {}
write results:              {}
results directory:          {}
results format:             {}
",
                page_results,
                write_results,
                results_directory.to_string_lossy(),
                results_format
            ),
            GraphQCommand::Query {
                page_results,
                query,
                benchmark,
                bench_num_runs,
                bench_num_warmup_runs,
                print_query,
                write_results,
                results_directory,
                results_format,
            } => {
                let benchmark_info = match benchmark {
                    true => Some(format!(
                        r#"
benchmark num runs:         {}
benchmark num warmup runs:  {}"#,
                        bench_num_runs, bench_num_warmup_runs,
                    )),
                    false => None,
                };

                let query_info = if query.as_str() == "-" {
                    "
query:                      -
"
                    .to_string()
                } else {
                    format!(
                        r#"
query:
---
{}
---
"#,
                        query
                    )
                };

                let output_info = if *write_results {
                    format!(
                        "
write results:              true
results directory:          {}
results format:             {}
",
                        results_directory.to_string_lossy(),
                        results_format
                    )
                } else {
                    r#"
write results:              false
"#
                    .to_string()
                };

                format!(
                    r#"
command:                    query
display results via pager:  {}
benchmark:                  {}{}
print query:                {}{}{}"#,
                    page_results,
                    benchmark,
                    benchmark_info.unwrap_or_default(),
                    print_query,
                    output_info,
                    query_info,
                )
            }
        };

        f.write_str(&output)
    }
}
