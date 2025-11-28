use serde_json::Value;
use tabled::builder::Builder;
use tabled::settings::style::Style;

pub fn print_results(results: &Value) {
    let results_array = match results {
        Value::Array(arr) => arr,
        Value::Object(obj) => match obj.get("results") {
            Some(Value::Array(arr)) => arr,
            _ => {
                if let Ok(json) = serde_json::to_string_pretty(results) {
                    println!("{}", json);
                }
                return;
            }
        },
        _ => {
            if let Ok(json) = serde_json::to_string_pretty(results) {
                println!("{}", json);
            }
            return;
        }
    };

    if results_array.is_empty() {
        println!("No results");
        return;
    }

    let mut builder = Builder::default();

    if let Some(Value::Object(first)) = results_array.first() {
        let headers: Vec<String> = first.keys().cloned().collect();
        builder.push_record(&headers);

        for result in results_array {
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

    println!("{}", table);
}
