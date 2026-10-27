use crate::Error;
use gltf_meshdoc::Error as GltfError;

/// The bridge's error as a meshconv [`Error::Format`].
impl From<GltfError> for Error {
    fn from(error: GltfError) -> Self {
        Error::Format(Box::new(error))
    }
}
