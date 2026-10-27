use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error decoding or encoding an SDF Json (`.sdfj` / `.sdfjz`) document.
#[derive(Debug)]
pub enum Error {
    /// A document's font failed to load.
    Font(String),

    /// The document JSON could not be parsed or written.
    Json(String),

    /// The `.sdfjz` archive was malformed.
    Invalid(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Font(message) | Error::Json(message) | Error::Invalid(message) => {
                write!(f, "{message}")
            }
        }
    }
}

impl StdError for Error {}
