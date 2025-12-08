use super::get_results;
use crate::config::DEFAULT_RESULTS_DIR;
use crate::domain::{OutputFormat, Pager, QueryResults};
use crate::repository::QueryExecutor;
use crate::service::{page_results, write_results};
use anyhow::Context;
use chrono::Utc;
use colored::Colorize;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

const BANNER: &str = include_str!("assets/logo.txt");
const COMMANDS: &str = include_str!("assets/commands.txt");
const KEYMAPS: &str = include_str!("assets/keymaps.txt");

pub struct ConsoleConfig {
    pub page_results: bool,
    pub write_results: bool,
    pub results_directory: PathBuf,
    pub results_format: OutputFormat,
}

pub struct Console<D: QueryExecutor> {
    db_client: D,
    history_file_path: PathBuf,
    config: ConsoleConfig,
    pager: Option<Pager>,
}

#[allow(unused)]
enum ConsoleColor {
    Blue,
    Yellow,
    Green,
}

impl<D: QueryExecutor> Console<D> {
    pub fn new(
        db_client: D,
        history_file_path: PathBuf,
        config: ConsoleConfig,
        pager: Option<Pager>,
    ) -> Self {
        Self {
            db_client,
            history_file_path,
            config,
            pager,
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
                cmd if cmd.starts_with("page") => match cmd.split_once(" ") {
                    Some((_, "on")) => {
                        if self.pager.is_none() {
                            match crate::utils::get_pager() {
                                Ok(p) => {
                                    self.pager = Some(p);
                                    self.config.page_results = true;
                                    print_info("paging results turned ON");
                                }
                                Err(e) => {
                                    print_error(format!("Error: couldn't turn on pager: {:#}", e));
                                }
                            }
                        } else if !self.config.page_results {
                            self.config.page_results = true;
                            print_info("paging results turned ON");
                        }
                    }
                    Some((_, "off")) => {
                        self.config.page_results = false;
                        print_info("paging results turned OFF");
                    }
                    _ => print_error("Usage: page on/off"),
                },
                cmd if cmd.starts_with("format") => match cmd.split_once(" ") {
                    Some((_, arg)) => match OutputFormat::from_str(arg) {
                        Ok(f) => {
                            self.config.results_format = f;
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
                        self.config.results_directory = PathBuf::new().join(DEFAULT_RESULTS_DIR);
                        print_info(format!(
                            "output path changed to gcue's default: {}",
                            DEFAULT_RESULTS_DIR
                        ));
                    }
                    Some((_, arg)) => match PathBuf::from_str(arg) {
                        Ok(p) => {
                            self.config.results_directory = p;
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
                        self.config.write_results = true;
                        print_info("writing output turned ON");
                    }
                    Some((_, "off")) => {
                        self.config.write_results = false;
                        print_info("writing output turned OFF");
                    }
                    _ => print_error("Usage: write on/off"),
                },
                q => {
                    if let Err(e) = editor.add_history_entry(q) {
                        println!("Error: {e}");
                    }

                    match self.db_client.execute_query(q).await {
                        Ok(QueryResults::Empty) => {
                            println!("\nNo results\n");
                        }
                        Ok(QueryResults::NonEmpty(results)) => {
                            if self.config.write_results {
                                match write_results(
                                    &results,
                                    &self.config.results_directory,
                                    &self.config.results_format,
                                    Utc::now(),
                                ) {
                                    Ok(p) => {
                                        print_info(format!(
                                            "wrote results to {}",
                                            p.to_string_lossy()
                                        ));

                                        if self.config.page_results
                                            && let Some(pager) = &self.pager
                                            && let Err(e) = page_results(&p, pager)
                                        {
                                            print_error(format!(
                                                "Error: couldn't display results via pager: {:#}",
                                                e
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        print_error(format!(
                                            "Error: couldn't write results: {:#}",
                                            e
                                        ));
                                    }
                                }
                            } else if self.config.page_results
                                && let Some(pager) = &self.pager
                            {
                                let temp_results_directory = tempfile::tempdir().context(
                                    "couldn't create temporary directory for paging results",
                                )?;

                                match write_results(
                                    &results,
                                    &temp_results_directory,
                                    &self.config.results_format,
                                    Utc::now(),
                                ) {
                                    Ok(p) => {
                                        if let Err(e) = page_results(&p, pager) {
                                            print_error(format!(
                                                "Error: couldn't display results via pager: {:#}",
                                                e
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        print_error(format!(
                                            "Error: couldn't write results to temporary directory: {:#}",
                                            e
                                        ));
                                    }
                                }
                            } else {
                                let results_str = get_results(&results);
                                println!("\n{}\n", results_str);
                            }
                        }
                        Err(e) => print_error(format!("Error: couldn't get results: {:#}", e)),
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
   page results                   {}
   write results to filesystem    {}
   output format                  {}
   output path                    {}",
        if config.page_results { "ON" } else { "OFF" },
        if config.write_results { "ON" } else { "OFF" },
        config.results_format,
        config.results_directory.to_string_lossy(),
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
            page_results: false,
            results_format: OutputFormat::Csv,
            results_directory: PathBuf::new().join(DEFAULT_RESULTS_DIR),
            write_results: false,
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
