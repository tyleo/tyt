use crate::ParseFailure;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Range,
};

/// An error from the language pipeline.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// The text broke a token or grammar rule over the named byte range.
    Parse {
        range: Range<usize>,
        failure: ParseFailure,
    },
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Parse { range, failure } => {
                write!(
                    formatter,
                    "at bytes {}..{}: {failure}",
                    range.start, range.end
                )
            }
        }
    }
}

impl StdError for Error {}
