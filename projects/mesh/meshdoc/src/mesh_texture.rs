use crate::{BMeshImage, MeshMagFilter, MeshMinFilter, MeshWrap};
use branded_id::U32Id;

/// A texture. The id references a [`MeshMain`](crate::MeshMain) and is
/// meaningful only within it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeshTexture {
    /// The image sampled.
    pub image_id: U32Id<BMeshImage>,

    /// The magnification filter, or `None` to leave it to the renderer.
    pub mag_filter: Option<MeshMagFilter>,

    /// The minification filter, or `None` to leave it to the renderer.
    pub min_filter: Option<MeshMinFilter>,

    /// The wrap along the `u` axis.
    pub wrap_s: MeshWrap,

    /// The wrap along the `v` axis.
    pub wrap_t: MeshWrap,
}

impl MeshTexture {
    /// A texture of `image_id` with no filters set and both axes repeating.
    pub fn new(image_id: U32Id<BMeshImage>) -> Self {
        Self {
            image_id,
            mag_filter: None,
            min_filter: None,
            wrap_s: MeshWrap::Repeat,
            wrap_t: MeshWrap::Repeat,
        }
    }
}
