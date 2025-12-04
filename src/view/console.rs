use super::get_results;
use crate::repository::QueryExecutor;
use anyhow::Context;
use colored::Colorize;
use rustyline::config::Configurer;
use std::io::Write;
use std::path::PathBuf;

const BANNER: &str = include_str!("assets/logo.txt");
const COMMANDS: &str = include_str!("assets/commands.txt");
const KEYMAPS: &str = include_str!("assets/keymaps.txt");

pub struct Console<D: QueryExecutor> {
    db_client: D,
    history_file_path: PathBuf,
}

#[allow(unused)]
enum ConsoleColor {
    Blue,
    Yellow,
    Green,
}

impl<D: QueryExecutor> Console<D> {
    pub fn new(db_client: D, history_file_path: PathBuf) -> Self {
        Self {
            db_client,
            history_file_path,
        }
    }

    pub async fn run_loop(&self) -> anyhow::Result<()> {
        print_banner(std::io::stdout(), true);
        print_help(std::io::stdout(), &self.db_client.db_uri(), true);

        let mut editor = rustyline::DefaultEditor::new()?;
        editor.set_completion_type(rustyline::CompletionType::Circular);
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
                    continue;
                }
                "help" | ":h" => {
                    print_help(std::io::stdout(), &self.db_client.db_uri(), true);
                    continue;
                }
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

fn print_banner(mut writer: impl Write, color: bool) {
    if color {
        let _ = writeln!(writer, "{}\n", BANNER.blue());
    } else {
        let _ = writeln!(writer, "{}\n", BANNER);
    }
}

fn print_help(mut writer: impl Write, db_uri: &str, color: bool) {
    let help = if color {
        format!(
            r#" connected to: {}

{}
{}
"#,
            db_uri.cyan(),
            COMMANDS.yellow(),
            KEYMAPS.green()
        )
    } else {
        format!(
            r#" connected to: {}

{}
{}
"#,
            db_uri, COMMANDS, KEYMAPS,
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

        // WHEN
        print_banner(&mut buf, false);
        print_help(
            &mut buf,
            "https://db.cluster-cf0abc1xyzjk.us-east-1.neptune.amazonaws.com:8182",
            false,
        );

        // THEN
        let result = String::from_utf8(buf).expect("string should've been built");
        assert_snapshot!(result);
    }
}
