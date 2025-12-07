use crate::domain::{NonEmptyResults, OutputFormat};
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn write_results<P>(
    results: &NonEmptyResults,
    results_directory: P,
    format: &OutputFormat,
    reference_time: DateTime<Utc>,
) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path>,
{
    std::fs::create_dir_all(&results_directory).with_context(|| {
        format!(
            "couldn't create results directory: {}",
            results_directory.as_ref().to_string_lossy()
        )
    })?;

    let file_name = reference_time.format("%Y-%m-%d-%H-%M-%S");
    let results_file_path =
        results_directory
            .as_ref()
            .join(format!("{}.{}", file_name, format.extension()));

    let file = File::create(&results_file_path).with_context(|| {
        format!(
            "couldn't create results file: {}",
            results_file_path.to_string_lossy()
        )
    })?;

    match format {
        OutputFormat::Csv => write_csv(results, file)?,
        OutputFormat::Json => write_json(results, file)?,
    }

    Ok(results_file_path)
}

fn write_csv<W>(results: &NonEmptyResults, writer: W) -> anyhow::Result<()>
where
    W: Write,
{
    let mut csv_writer = csv::Writer::from_writer(writer);

    let Some(first) = results.first().as_object() else {
        // this is alright as the result from the db is expected to be an array of objects, each
        // having the same keys
        anyhow::bail!("expected results to be an array of objects");
    };

    let headers: Vec<&str> = first.keys().map(|s| s.as_str()).collect();
    csv_writer.write_record(&headers)?;

    for result in results.list() {
        let Some(obj) = result.as_object() else {
            anyhow::bail!("expected each result to be an object");
        };

        let row: Vec<String> = headers
            .iter()
            .map(|&header| obj.get(header).map(value_to_csv_field).unwrap_or_default())
            .collect();

        csv_writer.write_record(&row)?;
    }

    csv_writer.flush()?;
    Ok(())
}

fn value_to_csv_field(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn write_json<W>(results: &NonEmptyResults, mut writer: W) -> anyhow::Result<()>
where
    W: Write,
{
    let json_string = serde_json::to_string_pretty(results.list())
        .context("couldn't serialize results to JSON")?;
    writer
        .write_all(json_string.as_bytes())
        .context("couldn't write bytes to file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn write_csv_writes_correct_headers_and_rows() -> anyhow::Result<()> {
        // GIVEN
        let results = results_sample_one();
        let mut buffer = Vec::new();

        // WHEN
        write_csv(&results, &mut buffer)?;

        // THEN
        let result = String::from_utf8(buffer)?;
        assert_snapshot!(result);

        Ok(())
    }

    #[test]
    fn write_csv_handles_non_strings_and_numbers() -> anyhow::Result<()> {
        // GIVEN
        let results = results_sample_two();
        let mut buffer = Vec::new();

        // WHEN
        write_csv(&results, &mut buffer)?;

        // THEN
        let result = String::from_utf8(buffer)?;
        assert_snapshot!(result);

        Ok(())
    }

    #[test]
    fn write_json_works_as_expected() -> anyhow::Result<()> {
        // GIVEN
        let results = results_sample_one();
        let mut buffer = Vec::new();

        // WHEN
        write_json(&results, &mut buffer)?;

        // THEN
        let result = String::from_utf8(buffer)?;
        assert_snapshot!(result);

        Ok(())
    }

    fn results_sample_one() -> NonEmptyResults {
        let results = vec![
            serde_json::json!({"language": "Rust", "creator": "Graydon Hoare", "year": 2010}),
            serde_json::json!({"language": "Python", "creator": "Guido van Rossum", "year": 1991}),
            serde_json::json!({"language": "Go", "creator": "Rob Pike", "year": 2009}),
        ];

        NonEmptyResults::try_from(results).expect("results should've been created")
    }

    fn results_sample_two() -> NonEmptyResults {
        let results = vec![
            serde_json::json!(
            {
              "language": "Rust",
              "creators": ["Graydon Hoare"],
              "year": 2010,
              "compiled": true,
              "features": {
                "garbage_collection": false,
                "static_typing": true
              }
            }),
            serde_json::json!(
            {
              "language": "Go",
              "creators": ["Robert Griesemer", "Rob Pike", "Ken Thompson", null],
              "year": 2009,
              "compiled": true,
              "features": {
                "garbage_collection": true,
                "static_typing": true
              }
            }),
            serde_json::json!(
            {
              "language": "Python",
              "creator": null,
              "year": 1991,
              "compiled": false,
              "features": {
                "garbage_collection": true,
                "static_typing": null
              }
            }),
            serde_json::json!(
            {
              "language": "Gleam",
              "creators": ["Louis Pilfold"],
              "year": 2016,
              "compiled": true,
              "features": null
            }),
        ];

        NonEmptyResults::try_from(results).expect("results should've been created")
    }
}
