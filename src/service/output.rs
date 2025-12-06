use crate::domain::OutputFormat;
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn write_results<P>(
    results: &[Value],
    results_directory: P,
    format: &OutputFormat,
    reference_time: DateTime<Utc>,
) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path>,
{
    tokio::fs::create_dir_all(&results_directory)
        .await
        .with_context(|| {
            format!(
                "failed to create results directory: {}",
                results_directory.as_ref().to_string_lossy()
            )
        })?;

    let file_name = reference_time.format("%b-%d-%H-%M-%S");
    let output_file_path = match format {
        OutputFormat::Csv => todo!(),
        OutputFormat::Json => results_directory
            .as_ref()
            .join(format!("{}.json", file_name)),
    };

    let file = tokio::fs::File::create(&output_file_path)
        .await
        .with_context(|| {
            format!(
                "couldn't open output file: {}",
                output_file_path.to_string_lossy()
            )
        })?;

    match format {
        OutputFormat::Csv => todo!(),
        OutputFormat::Json => write_json(results, file).await?,
    }

    Ok(output_file_path)
}

async fn write_json<W>(results: &[Value], mut writer: W) -> anyhow::Result<()>
where
    W: tokio::io::AsyncWrite + Unpin,
{
    let json_string =
        serde_json::to_string_pretty(results).context("couldn't serialize results to JSON")?;
    writer
        .write_all(json_string.as_bytes())
        .await
        .context("couldn't write byes to file")?;

    Ok(())
}
