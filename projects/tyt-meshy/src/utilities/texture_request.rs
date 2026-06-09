use crate::TextureInput;
use std::path::PathBuf;

/// A fully-resolved retexture create request: the serializable [`TextureInput`]
/// stored in the task file, plus the filesystem path of the style image sent
/// inline as base64 in the API request.
#[derive(Clone, Debug)]
pub struct TextureRequest {
    /// The input as written to the task file.
    pub input: TextureInput,

    /// The filesystem path of the style image, if any, read and base64-encoded
    /// as the API's `image_style_url`.
    pub image_style_path: Option<PathBuf>,
}
