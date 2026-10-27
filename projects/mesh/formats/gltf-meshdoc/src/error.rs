use gltf::Error as GltfError;
use meshdoc::Error as MeshError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};

/// An error from gltf-meshdoc.
#[derive(Debug)]
pub enum Error {
    /// Parsing or validating glTF JSON, or framing a GLB, failed.
    Codec(GltfError),

    /// Mesh data was readable but semantically malformed.
    Invalid(String),

    /// A meshdoc construction, mutation, or insertion was rejected.
    Mesh(MeshError),
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
            Error::Codec(error) => error.fmt(f),
            Error::Invalid(message) => write!(f, "{message}"),
            Error::Mesh(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Codec(error) => Some(error),
            Error::Invalid(_) => None,
            Error::Mesh(error) => Some(error),
        }
    }
}

impl From<GltfError> for Error {
    fn from(error: GltfError) -> Self {
        Error::Codec(error)
    }
}

impl From<MeshError> for Error {
    fn from(error: MeshError) -> Self {
        Error::Mesh(error)
    }
}
