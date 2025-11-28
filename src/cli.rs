use clap::{Parser, Subcommand};

const NOT_PROVIDED: &str = "<NOT PROVIDED>";

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
    /// Run gcue's console or execute a one-off query
    #[command(name = "run")]
    Run {
        /// Cypher query to execute; If not provided, starts interactive console
        #[arg(long = "query", short = 'q')]
        query: Option<String>,
    },
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match &self.command {
            GraphQCommand::Run { query } => {
                format!(
                    r#"
command:    Run
query:
{}
"#,
                    query.as_deref().unwrap_or(NOT_PROVIDED),
                )
            }
        };

        f.write_str(&output)
    }
}
