pub enum OutputFormat {
    Csv,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            OutputFormat::Csv => "csv",
            OutputFormat::Json => "json",
        };

        write!(f, "{}", value)
    }
}
