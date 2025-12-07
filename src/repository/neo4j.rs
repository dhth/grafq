use anyhow::Context;
use neo4rs::{ConfigBuilder, Graph, query as neo4j_query};
use serde_json::Value;

use crate::domain::QueryResults;

pub struct Neo4jClient {
    inner: Graph,
    db_uri: String,
}

pub struct Neo4jConfig {
    pub db_uri: String,
    pub user: String,
    pub password: String,
    pub database_name: String,
}

impl Neo4jClient {
    pub async fn new(config: &Neo4jConfig) -> anyhow::Result<Self> {
        let cfg = ConfigBuilder::default()
            .uri(config.db_uri.as_str())
            .user(config.user.as_str())
            .password(config.password.as_str())
            .db(config.database_name.as_str())
            .build()?;

        let graph = Graph::connect(cfg).await?;

        Ok(Self {
            inner: graph,
            db_uri: config.db_uri.to_string(),
        })
    }

    pub(super) fn db_uri(&self) -> String {
        self.db_uri.clone()
    }

    pub(super) async fn execute_query(&self, query: &str) -> anyhow::Result<QueryResults> {
        let mut result = self
            .inner
            .execute(neo4j_query(query))
            .await
            .context("couldn't execute query")?;

        let mut results = Vec::new();

        while let Some(row) = result
            .next()
            .await
            .context("couldn't get row from results")?
        {
            let row_value = row.to::<Value>().context("couldn't parse row as value")?;
            results.push(row_value);
        }

        Ok(results.into())
    }
}
