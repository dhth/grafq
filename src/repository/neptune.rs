use anyhow::Context;
use aws_config::SdkConfig;
use aws_sdk_neptunedata::Client as NeptuneDataClient;
use aws_smithy_types::{Document, Number};
use serde_json::{Map, Value};

use crate::domain::QueryResults;

pub struct NeptuneClient {
    inner: NeptuneDataClient,
    db_uri: String,
}

impl NeptuneClient {
    pub fn new(sdk_config: &SdkConfig, db_uri: &str) -> Self {
        let neptune_config = aws_sdk_neptunedata::config::Builder::from(sdk_config)
            .endpoint_url(db_uri)
            .build();
        let neptune_client = aws_sdk_neptunedata::Client::from_conf(neptune_config);

        Self {
            inner: neptune_client,
            db_uri: db_uri.to_string(),
        }
    }

    pub(super) fn db_uri(&self) -> String {
        self.db_uri.clone()
    }

    pub(super) async fn execute_query(&self, query: &str) -> anyhow::Result<QueryResults> {
        let output = self
            .inner
            .execute_open_cypher_query()
            .open_cypher_query(query)
            .send()
            .await
            .context("couldn't execute query")?;

        let document = output.results();

        let result_value = document_to_value(document);

        let results = match result_value {
            Value::Array(arr) => arr,
            _ => anyhow::bail!("unexpected response received, was expecting an array"),
        };

        Ok(results.into())
    }
}

fn document_to_value(doc: &Document) -> Value {
    match doc {
        Document::Object(map) => {
            let mut obj = Map::new();
            for (key, value) in map {
                obj.insert(key.clone(), document_to_value(value));
            }
            Value::Object(obj)
        }
        Document::Array(items) => {
            let arr = items.iter().map(document_to_value).collect();
            Value::Array(arr)
        }
        Document::String(s) => Value::String(s.clone()),
        Document::Bool(b) => Value::Bool(*b),
        Document::Null => Value::Null,
        Document::Number(num) => number_to_value(*num),
    }
}

fn number_to_value(num: Number) -> Value {
    match num {
        Number::PosInt(u) => Value::Number(u.into()),
        Number::NegInt(i) => Value::Number(i.into()),
        Number::Float(f) => serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_yaml_snapshot;
    use std::collections::HashMap;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn document_to_value_handles_all_primitive_types() {
        // GIVEN
        let doc = get_document_with_numbers(vec![
            ("pos_int_42", Number::PosInt(42)),
            ("pos_int_zero", Number::PosInt(0)),
            ("pos_int_max", Number::PosInt(u64::MAX)),
            ("neg_int_minus_42", Number::NegInt(-42)),
            ("neg_int_minus_1", Number::NegInt(-1)),
            ("neg_int_min", Number::NegInt(i64::MIN)),
            ("pos_float", Number::Float(2.71)),
            ("neg_float", Number::Float(-2.71)),
            ("not_a_number", Number::Float(f64::NAN)),
            ("pos_infinity", Number::Float(f64::INFINITY)),
            ("neg_infinity", Number::Float(f64::NEG_INFINITY)),
        ]);

        // WHEN
        let result = document_to_value(&doc);

        // THEN
        assert_yaml_snapshot!(result, @r"
        neg_float: -2.71
        neg_infinity: ~
        neg_int_min: -9223372036854775808
        neg_int_minus_1: -1
        neg_int_minus_42: -42
        not_a_number: ~
        pos_float: 2.71
        pos_infinity: ~
        pos_int_42: 42
        pos_int_max: 18446744073709551615
        pos_int_zero: 0
        ");
    }

    #[test]
    fn document_to_value_handles_string_and_boolean_types() {
        // GIVEN
        let mut map = HashMap::new();
        map.insert(
            "string".to_string(),
            Document::String("hello world".to_string()),
        );
        map.insert("empty_string".to_string(), Document::String(String::new()));
        map.insert("bool_true".to_string(), Document::Bool(true));
        map.insert("bool_false".to_string(), Document::Bool(false));
        map.insert("null_value".to_string(), Document::Null);
        let doc = Document::Object(map);

        // WHEN
        let result = document_to_value(&doc);

        // THEN
        assert_yaml_snapshot!(result, @r#"
        bool_false: false
        bool_true: true
        empty_string: ""
        null_value: ~
        string: hello world
        "#);
    }

    #[test]
    fn document_to_value_handles_empty_collections() {
        // GIVEN
        let values: Vec<_> = ["value-1", "value-2"]
            .iter()
            .map(|v| Document::String(v.to_string()))
            .collect();
        let doc = Document::Array(values);

        // WHEN
        let result = document_to_value(&doc);

        // THEN
        assert_yaml_snapshot!(result, @r"
        - value-1
        - value-2
        ");
    }

    #[test]
    fn document_to_value_handles_array_at_root() {
        // GIVEN
        let mut map = HashMap::new();
        map.insert("empty_object".to_string(), Document::Object(HashMap::new()));
        map.insert("empty_array".to_string(), Document::Array(vec![]));
        let doc = Document::Object(map);

        // WHEN
        let result = document_to_value(&doc);

        // THEN
        assert_yaml_snapshot!(result, @r"
        empty_array: []
        empty_object: {}
        ");
    }

    #[test]
    fn document_to_value_handles_nested_structures() {
        // GIVEN
        let mut inner_user = HashMap::new();
        inner_user.insert("id".to_string(), Document::Number(Number::PosInt(1)));
        inner_user.insert("active".to_string(), Document::Bool(true));

        let mut obj1 = HashMap::new();
        obj1.insert("id".to_string(), Document::Number(Number::PosInt(1)));
        obj1.insert("name".to_string(), Document::String("First".to_string()));

        let mut obj2 = HashMap::new();
        obj2.insert("id".to_string(), Document::Number(Number::PosInt(2)));
        obj2.insert("name".to_string(), Document::String("Second".to_string()));

        let mut level3 = HashMap::new();
        level3.insert("value".to_string(), Document::Number(Number::PosInt(999)));

        let mut level2 = HashMap::new();
        level2.insert("nested".to_string(), Document::Object(level3));

        let mut root = HashMap::new();
        root.insert("object_in_object".to_string(), Document::Object(inner_user));
        root.insert(
            "array_of_objects".to_string(),
            Document::Array(vec![Document::Object(obj1), Document::Object(obj2)]),
        );
        root.insert(
            "object_with_array".to_string(),
            Document::Object({
                let mut m = HashMap::new();
                m.insert(
                    "items".to_string(),
                    Document::Array(vec![
                        Document::String("a".to_string()),
                        Document::String("b".to_string()),
                    ]),
                );
                m
            }),
        );
        root.insert(
            "deep_nesting".to_string(),
            Document::Object({
                let mut m = HashMap::new();
                m.insert("level2".to_string(), Document::Object(level2));
                m
            }),
        );

        let doc = Document::Object(root);

        // WHEN
        let result = document_to_value(&doc);

        // THEN
        assert_yaml_snapshot!(result, @r"
        array_of_objects:
          - id: 1
            name: First
          - id: 2
            name: Second
        deep_nesting:
          level2:
            nested:
              value: 999
        object_in_object:
          active: true
          id: 1
        object_with_array:
          items:
            - a
            - b
        ");
    }

    fn get_document_with_numbers(values: Vec<(&str, Number)>) -> Document {
        let mut map = HashMap::new();
        for (key, value) in values {
            map.insert(key.to_string(), Document::Number(value));
        }

        Document::Object(map)
    }
}
