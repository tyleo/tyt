use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding `.qb`, `.qbt`, or `.qbcl` bytes.
#[derive(Debug)]
pub enum Error {
    /// The input ended before a value could be read.
    UnexpectedEof(String),

    /// A matrix's zlib stream could not be decompressed.
    Zlib(String),

    /// The input was well-framed but semantically malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::UnexpectedEof(message) => write!(f, "unexpected end of input: {message}"),
            Error::Zlib(message) => write!(f, "zlib decompression failed: {message}"),
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {}
