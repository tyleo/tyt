use crate::operations::voxelize::DecodedImage;
use meshdoc::MeshImageMediaType;

/// Decodes the encoded images of a mesh document, the textures the
/// per-texel sampler reads.
pub trait DecodeImage {
    /// The 8-bit RGBA pixels of the image `bytes` encode as `media_type`,
    /// or the reason they cannot be decoded.
    fn decode_image(
        &self,
        media_type: MeshImageMediaType,
        bytes: &[u8],
    ) -> Result<DecodedImage, String>;
}
