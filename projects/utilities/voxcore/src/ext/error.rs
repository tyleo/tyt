use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from the ext protocol.
#[derive(Debug)]
pub enum Error {
    /// An ext failed to encode to or decode from its block form.
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
