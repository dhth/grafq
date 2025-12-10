use crate::cmds::{ConsoleCmdError, QueryCmdError};
use crate::repository::DbClientError;
use etcetera::HomeDirError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("couldn't determine your home directory: {0}")]
    XdgError(#[from] HomeDirError),
    #[error("{0}")]
    InvalidCLIUsage(&'static str),
    #[error(transparent)]
    ConsoleCmdError(#[from] ConsoleCmdError),
    #[error(transparent)]
    QueryCmdError(#[from] QueryCmdError),
    #[error(transparent)]
    Uncategorised(#[from] anyhow::Error),
}

impl AppError {
    pub fn follow_up(&self) -> Option<String> {
        match self {
            AppError::XdgError(_) => None,
            AppError::InvalidCLIUsage(_) => None,
            AppError::ConsoleCmdError(e) => match e {
                ConsoleCmdError::CouldntBuildDbClient(e) => follow_up_db_client_error(e),
                ConsoleCmdError::Uncategorised(_) => None,
            },
            AppError::QueryCmdError(e) => match e {
                QueryCmdError::CouldntBuildDbClient(e) => follow_up_db_client_error(e),
                QueryCmdError::Uncategorised(_) => None,
            },
            AppError::Uncategorised(_) => None,
        }
    }

    pub fn is_unexpected(&self) -> bool {
        match self {
            AppError::XdgError(_) => true,
            AppError::InvalidCLIUsage(_) => false,
            AppError::ConsoleCmdError(_) => false,
            AppError::QueryCmdError(_) => false,
            AppError::Uncategorised(_) => false,
        }
    }
}

fn follow_up_db_client_error(err: &DbClientError) -> Option<String> {
    match err {
        DbClientError::CouldntReadEnvVar(_) => None,
        DbClientError::DBUriNotSet => Some(
            "
grf requires the environment variable DB_URI to be set. You can set it as follows:
- bolt://127.0.0.1:7687 (for neo4j)
- https://abc.xyz.us-east-1.neptune.amazonaws.com:8182 (for AWS Neptune)
"
            .trim()
            .into(),
        ),
        DbClientError::DBUriHasUnsupportedProtocol(_) => Some(
            "
Only 'bolt' and 'https' protocols are supported by grf.
Use bolt for neo4j, and https for AWS Neptune.
"
            .trim()
            .into(),
        ),
        DbClientError::DBUriIsInvalid(_) => Some(
            "
The URI needs to be in the form <protocol>://<host>:<port>. For example:
- bolt://127.0.0.1:7687 (for neo4j)
- https://abc.xyz.us-east-1.neptune.amazonaws.com:8182 (for AWS Neptune)
"
            .trim()
            .into(),
        ),
        DbClientError::Neo4jConnectionInfoMissing(_) => Some(
            "
The environment variables NEO4J_USER, NEO4J_PASSWORD, and NEO4J_DB need to be set when connecting
to a neo4j database (which was determined by the bolt protocol in DB_URI).
            "
            .trim()
            .into(),
        ),
        DbClientError::Uncategorized(_) => None,
    }
}
