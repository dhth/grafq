use serde_json::Value;
use tabled::builder::Builder;
use tabled::settings::style::Style;

use crate::domain::NonEmptyResults;

pub fn get_results(results: &NonEmptyResults) -> String {
    let mut builder = Builder::default();

    if let Value::Object(first) = results.first() {
        let headers: Vec<String> = first.keys().cloned().collect();
        builder.push_record(&headers);

        for result in results.list() {
            if let Value::Object(row) = result {
                let cells: Vec<String> = headers
                    .iter()
                    .map(|h| {
                        row.get(h)
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                Value::Null => "null".to_string(),
                                _ => v.to_string(),
                            })
                            .unwrap_or_else(|| "".to_string())
                    })
                    .collect();
                builder.push_record(cells);
            }
        }
    }

    let mut table = builder.build();

    table.with(Style::psql());

    table.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn get_results_returns_correct_table_for_array_of_objects() {
        // GIVEN
        let results = vec![
            serde_json::json!({"language": "Rust", "creator": "Graydon Hoare", "year": 2010}),
            serde_json::json!({"language": "Python", "creator": "Guido van Rossum", "year": 1991}),
            serde_json::json!({"language": "Go", "creator": "Rob Pike", "year": 2009}),
        ];
        let results = NonEmptyResults::try_from(results).expect("results should've been created");

        // WHEN
        let result = get_results(&results);

        // THEN
        assert_snapshot!(result, @r"
         creator          | language | year 
        ------------------+----------+------
         Graydon Hoare    | Rust     | 2010 
         Guido van Rossum | Python   | 1991 
         Rob Pike         | Go       | 2009
        ");
    }

    #[test]
    fn get_results_formats_null_values_correctly() {
        // GIVEN
        let results = vec![
            serde_json::json!({"language": "Rust", "creator": null}),
            serde_json::json!({"language": "Python", "creator": "Guido van Rossum"}),
        ];
        let results = NonEmptyResults::try_from(results).expect("results should've been created");

        // WHEN
        let result = get_results(&results);

        // THEN
        assert_snapshot!(result, @r"
         creator          | language 
        ------------------+----------
         null             | Rust     
         Guido van Rossum | Python
        ");
    }

    #[test]
    fn get_results_converts_non_string_values_to_string() {
        // GIVEN
        let results = vec![
            serde_json::json!({"version": "1.0", "stable": true, "downloads": 1000}),
            serde_json::json!({"version": "2.0", "stable": false, "downloads": 5000}),
        ];
        let results = NonEmptyResults::try_from(results).expect("results should've been created");

        // WHEN
        let result = get_results(&results);

        // THEN
        assert_snapshot!(result, @r"
         downloads | stable | version 
        -----------+--------+---------
         1000      | true   | 1.0     
         5000      | false  | 2.0
        ");
    }

    #[test]
    fn get_results_skips_non_object_array_elements() {
        // GIVEN
        let results = vec![
            serde_json::json!({"language": "Rust", "creator": "Graydon Hoare"}),
            serde_json::json!("invalid"),
            serde_json::json!({"language": "Python", "creator": "Guido van Rossum"}),
        ];
        let results = NonEmptyResults::try_from(results).expect("results should've been created");

        // WHEN
        let result = get_results(&results);

        // THEN
        assert_snapshot!(result, @r"
         creator          | language 
        ------------------+----------
         Graydon Hoare    | Rust     
         Guido van Rossum | Python
        ");
    }

    #[test]
    fn get_results_shows_empty_string_for_missing_columns() {
        // GIVEN
        let results = vec![
            serde_json::json!({"language": "Rust", "creator": "Graydon Hoare", "year": 2010}),
            serde_json::json!({"language": "Python", "creator": "Guido van Rossum"}),
            serde_json::json!({"language": "Go", "year": 2009}),
        ];
        let results = NonEmptyResults::try_from(results).expect("results should've been created");

        // WHEN
        let result = get_results(&results);

        // THEN
        assert_snapshot!(result, @r"
         creator          | language | year 
        ------------------+----------+------
         Graydon Hoare    | Rust     | 2010 
         Guido van Rossum | Python   |      
                          | Go       | 2009
        ");
    }
}
