use super::NeptuneClient;
use super::{Neo4jClient, Neo4jConfig};
use crate::domain::QueryResults;
use crate::utils::{EnvVarError, get_env_var};
use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_sdk_neptunedata::config::ProvideCredentials;

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

impl DbClient {
    pub async fn verify_connectivity(&self) -> anyhow::Result<()> {
        self.execute_query("RETURN 1")
            .await
            .with_context(|| format!("couldn't verify connection to {}", self.db_uri()))?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DbClientError {
    #[error(transparent)]
    CouldntReadEnvVar(#[from] EnvVarError),
    #[error("DB_URI is not set")]
    DBUriNotSet,
    #[error(r#"DB_URI has unsupported scheme: "{0}""#)]
    DBUriHasUnsupportedScheme(String),
    #[error(r#"DB_URI is invalid: "{0}""#)]
    DBUriIsInvalid(String),
    #[error(r#"environment variable "{0}" is missing"#)]
    Neo4jConnectionInfoMissing(String),
    #[error(transparent)]
    Uncategorised(#[from] anyhow::Error),
}

pub async fn get_db_client() -> Result<DbClient, DbClientError> {
    let db_uri = get_env_var("DB_URI")?.ok_or(DbClientError::DBUriNotSet)?;

    fn get_neo4j_env_var(key: &str) -> Result<String, DbClientError> {
        get_env_var(key)?.ok_or_else(|| DbClientError::Neo4jConnectionInfoMissing(key.to_string()))
    }

    let db_client = match db_uri.split_once("://") {
        Some(("https", _)) => {
            let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
            if let Some(provider) = sdk_config.credentials_provider() {
                provider
                    .provide_credentials()
                    .await
                    .context("couldn't fetch AWS credentials")?;
            }

            let neptune_client = NeptuneClient::new(&sdk_config, &db_uri);
            Ok(DbClient::Neptune(neptune_client))
        }
        Some(("bolt", _)) => {
            let user = get_neo4j_env_var("NEO4J_USER")?;
            let password = get_neo4j_env_var("NEO4J_PASSWORD")?;
            let database_name = get_neo4j_env_var("NEO4J_DB")?;

            let config = Neo4jConfig {
                db_uri,
                user,
                password,
                database_name,
            };

            let neo4j_client = Neo4jClient::new(&config).await?;
            Ok(DbClient::Neo4j(neo4j_client))
        }
        Some((scheme, _)) => Err(DbClientError::DBUriHasUnsupportedScheme(scheme.to_string())),
        None => Err(DbClientError::DBUriIsInvalid(db_uri)),
    }?;

    Ok(db_client)
}
