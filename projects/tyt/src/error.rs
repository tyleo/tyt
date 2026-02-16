use std::fmt;
use tyt_fbx::Error as FbxError;
use tyt_material::Error as MaterialError;

#[derive(Debug)]
pub enum Error {
    Fbx(FbxError),
    Material(MaterialError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Fbx(e) => e.fmt(f),
            Error::Material(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Fbx(e) => Some(e),
            Error::Material(e) => Some(e),
        }
    }
}

impl From<FbxError> for Error {
    fn from(e: FbxError) -> Self {
        Error::Fbx(e)
    }
}

impl From<MaterialError> for Error {
    fn from(e: MaterialError) -> Self {
        Error::Material(e)
    }
}
