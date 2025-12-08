use crate::domain::Pager;
use anyhow::Context;
use std::path::Path;

pub fn page_results<P>(results_file: P, pager: &Pager) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let mut cmd = pager.get_command();

    cmd.arg(results_file.as_ref())
        .spawn()
        .context("couldn't execute pager command")?
        .wait()
        .context("pager command failed")?;

    Ok(())
}
