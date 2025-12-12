use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ResultsFormat {
    Csv,
    Json,
}

impl ResultsFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ResultsFormat::Csv => "csv",
            ResultsFormat::Json => "json",
        }
    }
}

impl FromStr for ResultsFormat {
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

impl std::fmt::Display for ResultsFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

pub struct NonEmptyResults(Vec<Value>);

impl NonEmptyResults {
    pub fn list(&self) -> &[Value] {
        &self.0
    }

    pub fn first(&self) -> &Value {
        &self.0[0]
    }
}

#[cfg(test)]
impl TryFrom<Vec<Value>> for NonEmptyResults {
    type Error = &'static str;

    fn try_from(value: Vec<Value>) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err("list is empty");
        }

        Ok(Self(value))
    }
}

pub enum QueryResults {
    Empty,
    NonEmpty(NonEmptyResults),
}

impl From<Vec<Value>> for QueryResults {
    fn from(value: Vec<Value>) -> Self {
        if value.is_empty() {
            return QueryResults::Empty;
        }

        QueryResults::NonEmpty(NonEmptyResults(value))
    }
}
