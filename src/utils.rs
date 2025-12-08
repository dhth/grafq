use anyhow::Context;

use crate::domain::Pager;

pub fn get_pager() -> anyhow::Result<Pager> {
    let pager_env_var = get_env_var("GCUE_PAGER")?;
    let pager = match pager_env_var {
        Some(p) => Pager::custom(&p)?,
        None => Pager::default()?,
    };

    Ok(pager)
}

pub fn get_mandatory_env_var(key: &str) -> anyhow::Result<String> {
    get_env_var(key)?.context(format!("{} is not set", key))
}

pub fn get_env_var(key: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(key) {
        Ok(v) => Ok(Some(v)),
        Err(e) => match e {
            std::env::VarError::NotPresent => Ok(None),
            std::env::VarError::NotUnicode(_) => anyhow::bail!("{} is not valid unicode", key),
        },
    }
}
