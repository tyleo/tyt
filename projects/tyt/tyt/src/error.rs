use std::{error::Error as StdError, fmt};
use tyt_claude::Error as ClaudeError;
use tyt_cubemap::Error as CubemapError;
use tyt_fbx::Error as FbxError;
use tyt_fs::Error as FSError;
use tyt_image::Error as ImageError;
use tyt_material::Error as MaterialError;
use tyt_meshy::Error as MeshyError;
use tyt_meta::Error as MetaError;

use tyt_oai::Error as OAIError;
use tyt_vmax::Error as VMaxError;
use vxl::Error as VxlError;
#[derive(Debug)]
pub enum Error {
    Claude(ClaudeError),
    Cubemap(CubemapError),
    FS(FSError),
    Fbx(FbxError),
    Image(ImageError),
    Material(MaterialError),
    Meshy(MeshyError),
    Meta(MetaError),
    OAI(OAIError),
    VMax(VMaxError),
    Vxl(VxlError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Claude(e) => e.fmt(f),
            Error::Cubemap(e) => e.fmt(f),
            Error::FS(e) => e.fmt(f),
            Error::Fbx(e) => e.fmt(f),
            Error::Image(e) => e.fmt(f),
            Error::Material(e) => e.fmt(f),
            Error::Meshy(e) => e.fmt(f),
            Error::Meta(e) => e.fmt(f),
            Error::OAI(e) => e.fmt(f),
            Error::VMax(e) => e.fmt(f),
            Error::Vxl(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Claude(e) => Some(e),
            Error::Cubemap(e) => Some(e),
            Error::FS(e) => Some(e),
            Error::Fbx(e) => Some(e),
            Error::Image(e) => Some(e),
            Error::Material(e) => Some(e),
            Error::Meshy(e) => Some(e),
            Error::Meta(e) => Some(e),
            Error::OAI(e) => Some(e),
            Error::VMax(e) => Some(e),
            Error::Vxl(e) => Some(e),
        }
    }
}

impl From<ClaudeError> for Error {
    fn from(e: ClaudeError) -> Self {
        Error::Claude(e)
    }
}

impl From<CubemapError> for Error {
    fn from(e: CubemapError) -> Self {
        Error::Cubemap(e)
    }
}

impl From<FSError> for Error {
    fn from(e: FSError) -> Self {
        Error::FS(e)
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

impl From<MeshyError> for Error {
    fn from(e: MeshyError) -> Self {
        Error::Meshy(e)
    }
}

impl From<MetaError> for Error {
    fn from(e: MetaError) -> Self {
        Error::Meta(e)
    }
}

impl From<OAIError> for Error {
    fn from(e: OAIError) -> Self {
        Error::OAI(e)
    }
}

impl From<VMaxError> for Error {
    fn from(e: VMaxError) -> Self {
        Error::VMax(e)
    }
}

impl From<VxlError> for Error {
    fn from(e: VxlError) -> Self {
        Error::Vxl(e)
    }
}
