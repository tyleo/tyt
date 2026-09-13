use crate::{MeshImage, MeshImageMediaType, MeshImageSource};

/// The PNG signature.
pub const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// An image whose bytes carry the PNG signature and nothing else.
pub fn png_image() -> MeshImage {
    MeshImage {
        name: "atlas".to_owned(),
        media_type: MeshImageMediaType::Png,
        source: MeshImageSource::Bytes(PNG_SIGNATURE.to_vec()),
    }
}
