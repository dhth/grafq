use crate::domain::Pager;

pub fn get_pager() -> anyhow::Result<Pager> {
    let pager_env_var = get_env_var("GRF_PAGER")?;
    let pager = match pager_env_var {
        Some(p) => Pager::custom(&p)?,
        None => Pager::default()?,
    };

    Ok(pager)
}

#[derive(Debug, thiserror::Error)]
pub enum EnvVarError {
    #[error(r#"environment variable "{0}" is not valid unicode"#)]
    EnvVarIsInvalid(String),
}

pub fn get_env_var(key: &str) -> Result<Option<String>, EnvVarError> {
    match std::env::var(key) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            std::env::VarError::NotPresent => Ok(None),
            std::env::VarError::NotUnicode(_) => Err(EnvVarError::EnvVarIsInvalid(key.to_string())),
        },
    }
}
