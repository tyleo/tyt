use std::fmt;
use ty_fbx::Error as FbxError;

#[derive(Debug)]
pub enum Error {
    Fbx(FbxError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fbx(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Fbx(e) => Some(e),
        }
    }
}

impl From<FbxError> for Error {
    fn from(e: FbxError) -> Self {
        Error::Fbx(e)
    }
}
