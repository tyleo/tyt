use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding an object's voxel snapshots.
#[derive(Debug)]
pub enum Error {
    /// A snapshot is malformed. The message says which chunk and how.
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
