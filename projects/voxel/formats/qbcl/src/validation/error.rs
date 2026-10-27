use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error checking a decoded file.
#[derive(Debug)]
pub enum Error {
    /// A grid or image holds a different number of cells than its declared
    /// size needs. The message says which.
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
