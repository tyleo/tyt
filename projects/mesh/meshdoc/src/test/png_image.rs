use crate::{MeshImage, MeshImageMediaType, MeshImageSource, PNG_SIGNATURE};

/// An image whose bytes carry the PNG signature and nothing else.
pub fn png_image() -> MeshImage {
    MeshImage {
        name: "atlas".to_owned(),
        media_type: MeshImageMediaType::Png,
        source: MeshImageSource::Bytes(PNG_SIGNATURE.to_vec()),
    }
}
