use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

pub struct QueryFilenameCompleter {
    inner: FilenameCompleter,
}

impl Default for QueryFilenameCompleter {
    fn default() -> Self {
        Self {
            inner: FilenameCompleter::new(),
        }
    }
}

impl Completer for QueryFilenameCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        if !line.starts_with('@') {
            return Ok((pos, vec![]));
        }

        // Only complete when cursor is at the end
        if pos < line.len() {
            return Ok((pos, vec![]));
        }

        let path_segment = &line[1..pos];

        let (start, candidates) = self.inner.complete(path_segment, path_segment.len(), ctx)?;

        Ok((start + 1, candidates))
    }
}

impl Hinter for QueryFilenameCompleter {
    type Hint = String;
}

impl Highlighter for QueryFilenameCompleter {}

impl Validator for QueryFilenameCompleter {}

impl Helper for QueryFilenameCompleter {}
