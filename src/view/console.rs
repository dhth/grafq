use super::get_results;
use crate::domain::OutputFormat;
use crate::repository::QueryExecutor;
use anyhow::Context;
use colored::Colorize;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

const BANNER: &str = include_str!("assets/logo.txt");
const COMMANDS: &str = include_str!("assets/commands.txt");
const KEYMAPS: &str = include_str!("assets/keymaps.txt");
const DEFAULT_OUTPUT_PATH: &str = ".gcue";

impl FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        match trimmed {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err("invalid format provided; allowed values: [csv, json]"),
        }
    }
}

struct ConsoleConfig {
    format: OutputFormat,
    output_path: PathBuf,
    write: bool,
}

pub struct Console<D: QueryExecutor> {
    db_client: D,
    history_file_path: PathBuf,
    config: ConsoleConfig,
}

#[allow(unused)]
enum ConsoleColor {
    Blue,
    Yellow,
    Green,
}

impl<D: QueryExecutor> Console<D> {
    pub fn new(db_client: D, history_file_path: PathBuf) -> Self {
        let output_path = PathBuf::new().join(DEFAULT_OUTPUT_PATH);

        Self {
            db_client,
            history_file_path,
            config: ConsoleConfig {
                format: OutputFormat::Csv,
                output_path,
                write: false,
            },
        }
    }

    pub async fn run_loop(&mut self) -> anyhow::Result<()> {
        print_banner(std::io::stdout(), true);
        print_help(
            std::io::stdout(),
            &self.db_client.db_uri(),
            &self.config,
            true,
        );

        let mut editor = rustyline::DefaultEditor::new()?;
        let _ = editor.load_history(&self.history_file_path);

        loop {
            let query = editor.readline(">> ").context("couldn't read input")?;

            match query.trim() {
                "" => {}
                "bye" | "exit" | "quit" | ":q" => {
                    break;
                }
                "clear" => {
                    if editor.clear_screen().is_err() {
                        println!("{}", "Error: couldn't clear screen".red());
                    }
                }
                "help" | ":h" => {
                    print_help(
                        std::io::stdout(),
                        &self.db_client.db_uri(),
                        &self.config,
                        true,
                    );
                }
                cmd if cmd.starts_with("format") => match cmd.split_once(" ") {
                    Some((_, arg)) => match OutputFormat::from_str(arg) {
                        Ok(f) => {
                            self.config.format = f;
                            print_info(format!("output format set to: {}", arg));
                        }
                        Err(e) => {
                            print_error(e);
                        }
                    },
                    None => {
                        print_error("Usage: format <csv/json>");
                    }
                },
                cmd if cmd.starts_with("output") => match cmd.split_once(" ") {
                    Some((_, "reset")) => {
                        self.config.output_path = PathBuf::new().join(DEFAULT_OUTPUT_PATH);
                        print_info(format!(
                            "output path changed to gcue's default: {}",
                            DEFAULT_OUTPUT_PATH
                        ));
                    }
                    Some((_, arg)) => match PathBuf::from_str(arg) {
                        Ok(p) => {
                            self.config.output_path = p;
                            print_info(format!("output path changed to: {}", arg));
                        }
                        Err(e) => {
                            print_error(format!("Error: invalid path provided: {}", e));
                        }
                    },
                    None => print_error("Usage: output <PATH>"),
                },
                cmd if cmd.starts_with("write") => match cmd.split_once(" ") {
                    Some((_, "on")) => {
                        self.config.write = true;
                        print_info("writing output turned ON");
                    }
                    Some((_, "off")) => {
                        self.config.write = false;
                        print_info("writing output turned OFF");
                    }
                    _ => print_error("Usage: write on/off"),
                },
                q => {
                    if let Err(e) = editor.add_history_entry(q) {
                        println!("Error: {e}");
                    }
                    let value = self
                        .db_client
                        .execute_query(q)
                        .await
                        .context("couldn't execute query")?;

                    if let Some(results) = get_results(&value) {
                        println!("\n{}\n", results);
                    } else {
                        println!("\n {}\n", "no results".blue());
                    }
                }
            }
        }

        let _ = editor.save_history(&self.history_file_path);

        Ok(())
    }
}

fn print_error<S: AsRef<str>>(contents: S) {
    println!("{}", contents.as_ref().red());
}

fn print_info<S: AsRef<str>>(contents: S) {
    println!("{}", contents.as_ref().blue());
}

fn print_banner(mut writer: impl Write, color: bool) {
    if color {
        let _ = writeln!(writer, "{}\n", BANNER.blue());
    } else {
        let _ = writeln!(writer, "{}\n", BANNER);
    }
}

fn print_help(mut writer: impl Write, db_uri: &str, config: &ConsoleConfig, color: bool) {
    let config_help = format!(
        " config
   output format                  {}
   output path                    {}
   write output                   {}",
        config.format,
        config.output_path.to_string_lossy(),
        if config.write { "ON" } else { "OFF" }
    );

    let help = if color {
        format!(
            r#" connected to: {}

{}

{}
{}
"#,
            db_uri.cyan(),
            config_help.blue(),
            COMMANDS.yellow(),
            KEYMAPS.green()
        )
    } else {
        format!(
            r#" connected to: {}

{}

{}
{}
"#,
            db_uri, config_help, COMMANDS, KEYMAPS,
        )
    };

    let _ = write!(writer, "{}", help);
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn banner_and_help_are_printed_correctly() {
        // GIVEN
        let mut buf = Vec::new();
        let console_config = ConsoleConfig {
            format: OutputFormat::Csv,
            output_path: PathBuf::new().join(DEFAULT_OUTPUT_PATH),
            write: false,
        };

        // WHEN
        print_banner(&mut buf, false);
        print_help(
            &mut buf,
            "https://db.cluster-cf0abc1xyzjk.us-east-1.neptune.amazonaws.com:8182",
            &console_config,
            false,
        );

        // THEN
        let result = String::from_utf8(buf).expect("string should've been built");
        assert_snapshot!(result);
    }
}
