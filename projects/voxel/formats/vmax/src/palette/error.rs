use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error unpacking a palette's embedded color table.
#[derive(Debug)]
pub enum Error {
    /// The packed table is malformed. The message says how.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {}
