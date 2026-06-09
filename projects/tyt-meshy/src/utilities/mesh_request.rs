use crate::MeshInput;
use std::path::PathBuf;

/// A fully-resolved create request: the serializable [`MeshInput`] stored in the
/// task file, plus the filesystem paths of the images sent inline as base64 in
/// the API request.
#[derive(Clone, Debug)]
pub struct MeshRequest {
    /// The input as written to the task file.
    pub input: MeshInput,

    /// The filesystem path of the input image, read and base64-encoded as the
    /// API's `image_url`.
    pub image_path: PathBuf,

    /// The filesystem path of the texture guidance image, if any, read and
    /// base64-encoded as the API's `texture_image_url`.
    pub texture_image_path: Option<PathBuf>,
}
