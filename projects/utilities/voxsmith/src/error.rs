#[cfg(feature = "_gltf")]
use gltf::Error as GltfError;
#[cfg(feature = "_pathspec")]
use pathspec::Error as PathSpecError;
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
};
#[cfg(feature = "_treegrid")]
use treegrid::TreeGridError;
use voxcore::Error as VoxError;

/// An error from voxsmith.
#[derive(Debug)]
pub enum Error {
    /// Voxel data was readable but semantically malformed.
    Invalid(String),

    /// A voxcore construction, mutation, or insertion was rejected.
    Vox(VoxError),

    /// Reading a glTF or GLB mesh failed.
    #[cfg(feature = "_gltf")]
    Gltf(GltfError),

    /// A report layout rejected an option it does not consume.
    #[cfg(feature = "_treegrid")]
    TreeGrid(TreeGridError),

    /// A hierarchy-path pattern is not a valid gitignore-style glob.
    #[cfg(feature = "_pathspec")]
    PathSpec(PathSpecError),
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
            Error::Invalid(message) => write!(f, "{message}"),
            Error::Vox(error) => error.fmt(f),
            #[cfg(feature = "_gltf")]
            Error::Gltf(error) => error.fmt(f),
            #[cfg(feature = "_treegrid")]
            Error::TreeGrid(error) => error.fmt(f),
            #[cfg(feature = "_pathspec")]
            Error::PathSpec(error) => error.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Invalid(_) => None,
            Error::Vox(error) => Some(error),
            #[cfg(feature = "_gltf")]
            Error::Gltf(error) => Some(error),
            #[cfg(feature = "_treegrid")]
            Error::TreeGrid(error) => Some(error),
            #[cfg(feature = "_pathspec")]
            Error::PathSpec(error) => Some(error),
        }
    }
}

impl From<VoxError> for Error {
    fn from(error: VoxError) -> Self {
        Error::Vox(error)
    }
}

#[cfg(feature = "_gltf")]
impl From<GltfError> for Error {
    fn from(error: GltfError) -> Self {
        Error::Gltf(error)
    }
}

#[cfg(feature = "_treegrid")]
impl From<TreeGridError> for Error {
    fn from(error: TreeGridError) -> Self {
        Error::TreeGrid(error)
    }
}

#[cfg(feature = "_pathspec")]
impl From<PathSpecError> for Error {
    fn from(error: PathSpecError) -> Self {
        Error::PathSpec(error)
    }
}
