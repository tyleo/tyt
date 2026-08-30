use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding MagicaVoxel `.vox` bytes.
#[derive(Debug)]
pub enum Error {
    /// The input ended before a value could be read.
    UnexpectedEof(String),

    /// The input was well-framed but semantically malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::UnexpectedEof(message) => write!(f, "unexpected end of input: {message}"),
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {}
