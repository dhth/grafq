use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BenchmarkNumRuns(u16);

impl BenchmarkNumRuns {
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for BenchmarkNumRuns {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for BenchmarkNumRuns {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number: u16 = s.parse().map_err(|_| "value is not a valid number")?;

        if number == 0 {
            return Err("needs to be greater than 0");
        }

        Ok(BenchmarkNumRuns(number))
    }
}
