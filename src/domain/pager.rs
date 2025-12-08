use anyhow::Context;
use std::process::Command;

pub struct Pager(PagerInner);

impl Pager {
    pub fn default() -> anyhow::Result<Self> {
        let pager = PagerInner::Default;

        let binary = pager.binary();
        which::which(&binary).with_context(|| {
            format!(r#"couldn't find executable for gcue's default pager "{binary}""#)
        })?;

        Ok(Self(pager))
    }

    pub fn custom(cmd: &str) -> anyhow::Result<Self> {
        let pager = PagerInner::Custom(
            CustomPager::try_from(cmd)
                .with_context(|| format!(r#"couldn't build a pager from the command "{cmd}""#))?,
        );

        let binary = pager.binary();
        which::which(&binary)
            .with_context(|| format!(r#"couldn't find pager executable "{binary}""#))?;

        Ok(Self(pager))
    }
}

impl Pager {
    pub fn get_command(&self) -> Command {
        self.0.get_command()
    }
}

enum PagerInner {
    Default,
    Custom(CustomPager),
}

impl PagerInner {
    fn binary(&self) -> String {
        match self {
            Self::Default => "less".to_string(),
            Self::Custom(custom_pager) => custom_pager.binary.clone(),
        }
    }

    fn get_command(&self) -> Command {
        match self {
            PagerInner::Default => {
                let mut cmd = Command::new("less");
                cmd.arg("-+F");
                cmd
            }
            PagerInner::Custom(custom_pager) => {
                let mut cmd = Command::new(&custom_pager.binary);
                if !custom_pager.args.is_empty() {
                    cmd.args(&custom_pager.args);
                }
                cmd
            }
        }
    }
}

#[derive(Debug)]
struct CustomPager {
    binary: String,
    args: Vec<String>,
}

impl TryFrom<&str> for CustomPager {
    type Error = anyhow::Error;

    fn try_from(pager_cmd: &str) -> Result<Self, anyhow::Error> {
        if pager_cmd.is_empty() {
            anyhow::bail!("command is empty");
        }

        let parts = shlex::split(pager_cmd).ok_or(anyhow::anyhow!("couldn't parse command"))?;

        if parts.is_empty() {
            anyhow::bail!("command is empty");
        }

        Ok(Self {
            binary: parts[0].to_string(),
            args: parts[1..].to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use insta::assert_snapshot;

    //-------------//
    //  SUCCESSES  //
    //-------------//

    #[test]
    fn custom_pager_parses_pager_with_binary_only() -> anyhow::Result<()> {
        // GIVEN
        let pager_cmd = "bat";

        // WHEN
        let result = CustomPager::try_from(pager_cmd)?;

        // THEN
        assert_debug_snapshot!(result, @r#"
        CustomPager {
            binary: "bat",
            args: [],
        }
        "#);

        Ok(())
    }

    #[test]
    fn custom_pager_parses_pager_with_flags() -> anyhow::Result<()> {
        // GIVEN
        let pager_cmd = "bat -p --paging always";

        // WHEN
        let result = CustomPager::try_from(pager_cmd)?;

        // THEN
        assert_debug_snapshot!(result, @r#"
        CustomPager {
            binary: "bat",
            args: [
                "-p",
                "--paging",
                "always",
            ],
        }
        "#);

        Ok(())
    }

    //------------//
    //  FAILURES  //
    //------------//

    #[test]
    fn custom_pager_fails_to_parse_empty_pager_command() {
        // GIVEN
        // WHEN
        let result = CustomPager::try_from("  ").expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"command is empty");
    }

    #[test]
    fn custom_pager_fails_to_parse_malformed_pager_command() {
        // GIVEN
        let pager_cmd = "bat --paging 'unterminated";

        // WHEN
        let result = CustomPager::try_from(pager_cmd).expect_err("result should've been an error");

        // THEN
        assert_snapshot!(result, @"couldn't parse command");
    }
}
