mod neo4j;
mod neptune;

pub use neo4j::{Neo4jClient, Neo4jConfig};
pub use neptune::NeptuneClient;

use crate::domain::QueryResults;

pub trait QueryExecutor {
    async fn execute_query(&self, query: &str) -> anyhow::Result<QueryResults>;
    fn db_uri(&self) -> String;
}

pub enum DbClient {
    Neptune(NeptuneClient),
    Neo4j(Neo4jClient),
}

impl QueryExecutor for DbClient {
    async fn execute_query(&self, query: &str) -> anyhow::Result<QueryResults> {
        match self {
            DbClient::Neptune(c) => c.execute_query(query).await,
            DbClient::Neo4j(c) => c.execute_query(query).await,
        }
    }

    fn db_uri(&self) -> String {
        match self {
            DbClient::Neptune(c) => c.db_uri(),
            DbClient::Neo4j(c) => c.db_uri(),
        }
    }
}
