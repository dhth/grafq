use crate::repository::{DbClientError, get_db_client};
use crate::utils::get_pager;
use crate::view::{Console, ConsoleConfig};
use anyhow::Context;

#[derive(Debug, thiserror::Error)]
pub enum ConsoleCmdError {
    #[error("couldn't build db client")]
    CouldntBuildDbClient(#[from] DbClientError),
    #[error(transparent)]
    Uncategorised(#[from] anyhow::Error),
}

pub async fn handle_console_cmd(config: ConsoleConfig) -> Result<(), ConsoleCmdError> {
    let db_client = get_db_client().await?;
    db_client.verify_connectivity().await?;

    if let Some(parent) = config.history_file_path.parent() {
        tokio::fs::create_dir_all(parent).await.with_context(|| {
            format!(
                "couldn't create directory for grafq's history: {}",
                parent.display(),
            )
        })?;
    }

    let pager = if config.page_results {
        Some(get_pager()?)
    } else {
        None
    };

    let mut console = Console::new(db_client, config, pager);
    console.run_loop().await?;

    Ok(())
}
