use clap::{Parser, Subcommand};

use crate::domain::BenchmarkNumRuns;

/// gcue lets you query Neo4j/AWS Neptune databases via an interactive console
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
    /// Open gcue's console
    #[command()]
    Console,
    /// Execute a one-off query
    #[command()]
    Query {
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
        #[arg(short = 'p', long = "print-query")]
        print_query: bool,
    },
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match &self.command {
            GraphQCommand::Console => r#"
command:                    console
"#
            .to_string(),
            GraphQCommand::Query {
                query,
                benchmark,
                bench_num_runs,
                bench_num_warmup_runs,
                print_query,
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

                format!(
                    r#"
command:                    query
benchmark:                  {}{}
print query:                {}{}"#,
                    benchmark,
                    benchmark_info.unwrap_or_default(),
                    print_query,
                    query_info,
                )
            }
        };

        f.write_str(&output)
    }
}
