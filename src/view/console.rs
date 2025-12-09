use super::{QueryFilenameCompleter, get_results};
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
use std::time::Duration;
use tokio::time::Instant;

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

        let mut editor = rustyline::Editor::new()?;
        editor.set_helper(Some(QueryFilenameCompleter::default()));
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
                            "output path changed to grf's default: {}",
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
                user_input => {
                    if let Err(e) = editor.add_history_entry(user_input) {
                        println!("Error: {e}");
                    }

                    let query_to_execute = match get_query_from_user_input(user_input) {
                        Ok(q) => q,
                        Err(e) => {
                            print_error(format!("Error: {:#}", e));
                            continue;
                        }
                    };

                    let start = Instant::now();

                    let results = self.db_client.execute_query(&query_to_execute).await;
                    print_time(Instant::now().saturating_duration_since(start));

                    match results {
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

fn print_time(duration: Duration) {
    println!("{}", format!("took {} ms", duration.as_millis()).cyan());
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

fn get_query_from_user_input(contents: &str) -> anyhow::Result<String> {
    let query_to_execute = if let Some(file_path) = contents.strip_prefix('@').map(|p| p.trim()) {
        if file_path.is_empty() {
            anyhow::bail!("no file path provided after '@'");
        }

        let contents = std::fs::read_to_string(file_path)
            .with_context(|| format!(r#"couldn't read file "{}""#, file_path))?;

        match contents.trim() {
            "" => anyhow::bail!("file '{}' is empty", file_path),
            c => c.to_string(),
        }
    } else {
        contents.trim().to_string()
    };

    Ok(query_to_execute)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    const QUERY_FILE_PATH: &str = "src/view/testdata/query.cypher";
    const QUERY_WITH_WHITESPACE_FILE_PATH: &str = "src/view/testdata/query-with-whitespace.cypher";
    const EMPTY_QUERY_FILE_PATH: &str = "src/view/testdata/empty.cypher";

    //-------------//
    //  SUCCESSES  //
    //-------------//

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

    #[test]
    fn get_query_from_user_input_returns_query_as_is() -> anyhow::Result<()> {
        // GIVEN
        let input = "MATCH (n:Node) return n.id, n.name LIMIT 5;";

        // WHEN
        let result = get_query_from_user_input(input)?;

        // THEN
        assert_eq!(result, input);

        Ok(())
    }

    #[test]
    fn get_query_from_user_input_trims_whitespace_from_query() -> anyhow::Result<()> {
        // GIVEN
        let input = "  MATCH (n:Node) return n.id, n.name LIMIT 5;  ";

        // WHEN
        let result = get_query_from_user_input(input)?;

        // THEN
        assert_snapshot!(result, @"MATCH (n:Node) return n.id, n.name LIMIT 5;");

        Ok(())
    }

    #[test]
    fn get_query_from_user_input_reads_query_from_file() -> anyhow::Result<()> {
        // GIVEN
        let input = format!("@{}", QUERY_FILE_PATH);

        // WHEN
        let result = get_query_from_user_input(&input)?;

        // THEN
        assert_snapshot!(result, @"MATCH (n:Node) return n.id, n.name LIMIT 5;");

        Ok(())
    }

    #[test]
    fn get_query_from_user_input_trims_whitespace_in_file_contents() -> anyhow::Result<()> {
        // GIVEN
        let input = format!("@{}", QUERY_WITH_WHITESPACE_FILE_PATH);

        // WHEN
        let result = get_query_from_user_input(&input)?;

        // THEN
        assert_snapshot!(result, @"MATCH (n:Node) return n.id, n.name LIMIT 5;");

        Ok(())
    }

    #[test]
    fn get_query_from_user_input_trims_whitespace_in_file_path() -> anyhow::Result<()> {
        // GIVEN
        let input = format!("@  {}  ", QUERY_FILE_PATH);

        // WHEN
        let result = get_query_from_user_input(&input)?;

        // THEN
        assert_snapshot!(result, @"MATCH (n:Node) return n.id, n.name LIMIT 5;");

        Ok(())
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn get_query_from_user_input_fails_if_no_file_path_provided() {
        // GIVEN
        let input = "@";

        // WHEN
        let result = get_query_from_user_input(input).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"no file path provided after '@'");
    }

    #[test]
    fn get_query_from_user_input_fails_for_empty_file() {
        // GIVEN
        let input = format!("@{}", EMPTY_QUERY_FILE_PATH);

        // WHEN
        let result = get_query_from_user_input(&input).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"file 'src/view/testdata/empty.cypher' is empty");
    }

    #[test]
    fn get_query_from_user_input_fails_for_nonexistent_file() {
        // GIVEN
        let input = "@/nonexistent/path/to/query.cypher";

        // WHEN
        let result = get_query_from_user_input(input).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @r#"couldn't read file "/nonexistent/path/to/query.cypher""#);
    }
}
