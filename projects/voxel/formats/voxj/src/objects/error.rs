use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding or encoding an object's voxel-position or voxel-sample
/// blocks.
#[derive(Debug)]
pub enum Error {
    /// A block is malformed. The message says which block and how.
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
