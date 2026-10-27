use sdfcore::Error as SdfError;
#[cfg(feature = "codec")]
use sdfj_codec::Error as CodecError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from sdfj-sdfcore.
#[derive(Debug)]
pub enum Error {
    /// Reading document bytes failed.
    #[cfg(feature = "codec")]
    Codec(CodecError),

    /// The document was readable but holds what sdfcore cannot.
    Invalid(String),

    /// The state broke a rule [`SdfMain::new`](sdfcore::SdfMain::new) checks.
    Sdf(SdfError),
}

impl Error {
    /// Builds an [`Error::Invalid`] from a message.
    pub(crate) fn invalid(message: impl Display) -> Self {
        Error::Invalid(message.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            #[cfg(feature = "codec")]
            Error::Codec(error) => error.fmt(f),

            Error::Invalid(message) => write!(f, "{message}"),

            Error::Sdf(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            #[cfg(feature = "codec")]
            Error::Codec(error) => Some(error),

            Error::Invalid(_) => None,

            Error::Sdf(error) => Some(error),
        }
    }
}

#[cfg(feature = "codec")]
impl From<CodecError> for Error {
    fn from(error: CodecError) -> Self {
        Error::Codec(error)
    }
}

impl From<SdfError> for Error {
    fn from(error: SdfError) -> Self {
        Error::Sdf(error)
    }
}
