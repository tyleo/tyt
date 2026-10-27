use crate::{BMeshTexture, BMeshUvStream};
use branded_id::U32Id;

/// A reference to a texture as a material or property draws it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MeshTextureRef {
    /// The texture sampled.
    pub texture_id: U32Id<BMeshTexture>,

    /// The UV stream of each primitive drawing the material that the
    /// texture is sampled through.
    pub uv_stream_id: U32Id<BMeshUvStream>,
}
