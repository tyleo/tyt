use std::fmt;
use tyt_cubemap::Error as CubemapError;
use tyt_fbx::Error as FbxError;
use tyt_image::Error as ImageError;
use tyt_material::Error as MaterialError;

#[derive(Debug)]
pub enum Error {
    Cubemap(CubemapError),
    Fbx(FbxError),
    Image(ImageError),
    Material(MaterialError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Cubemap(e) => e.fmt(f),
            Error::Fbx(e) => e.fmt(f),
            Error::Image(e) => e.fmt(f),
            Error::Material(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Cubemap(e) => Some(e),
            Error::Fbx(e) => Some(e),
            Error::Image(e) => Some(e),
            Error::Material(e) => Some(e),
        }
    }
}

impl From<CubemapError> for Error {
    fn from(e: CubemapError) -> Self {
        Error::Cubemap(e)
    }
}

impl From<FbxError> for Error {
    fn from(e: FbxError) -> Self {
        Error::Fbx(e)
    }
}

impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        Error::Image(e)
    }
}

impl From<MaterialError> for Error {
    fn from(e: MaterialError) -> Self {
        Error::Material(e)
    }
}
