use serde_json::Value;

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
