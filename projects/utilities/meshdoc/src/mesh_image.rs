use crate::{MeshImageMediaType, MeshImageSource};

/// An encoded image. The document never decodes pixels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshImage {
    /// Display name.
    pub name: String,

    /// The encoding of the bytes.
    pub media_type: MeshImageMediaType,

    /// Where the bytes come from.
    pub source: MeshImageSource,
}
