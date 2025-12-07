use std::str::FromStr;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Csv,
    Json,
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Csv => "csv",
            OutputFormat::Json => "json",
        }
    }
}

impl FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        match trimmed {
            "csv" => Ok(Self::Csv),
            "json" => Ok(Self::Json),
            _ => Err("invalid format provided; allowed values: [csv, json]"),
        }
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}
