use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// An error from this crate.
#[derive(Debug)]
pub enum Error {
    /// An I/O error.
    IO(IOError),

    /// No OpenAI API key was configured.
    ApiKeyNotConfigured,

    /// No message was provided, on the command line or via stdin.
    NoMessage,

    /// The request could not be sent or its response could not be received.
    Http(String),

    /// OpenAI returned an error response.
    Api(String),

    /// The conversation could not be continued in place because the previous
    /// response is no longer cached by OpenAI.
    PreviousResponseExpired,

    /// The OpenAI response could not be interpreted.
    InvalidResponse(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::ApiKeyNotConfigured => f.write_str(
                "no OpenAI API key configured; add an \"oai\" section with an \"apiKey\" \
                 to a .tytusrconfig file at your git root",
            ),
            Error::NoMessage => {
                f.write_str("no message provided; pass a message argument or pipe one via stdin")
            }
            Error::Http(e) => write!(f, "OpenAI request failed: {e}"),
            Error::Api(e) => write!(f, "OpenAI API error: {e}"),
            Error::PreviousResponseExpired => f.write_str(
                "the previous response is no longer cached by OpenAI, so the conversation \
                 cannot be continued in place; re-run with --continue-kind last-image-all-text \
                 (or all-images-all-text / last-image-only) to rebuild the conversation locally",
            ),
            Error::InvalidResponse(e) => write!(f, "unexpected OpenAI response: {e}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IO(e) => Some(e),
            Error::ApiKeyNotConfigured
            | Error::NoMessage
            | Error::Http(_)
            | Error::Api(_)
            | Error::PreviousResponseExpired
            | Error::InvalidResponse(_) => None,
        }
    }
}

impl From<IOError> for Error {
    fn from(e: IOError) -> Self {
        Error::IO(e)
    }
}
