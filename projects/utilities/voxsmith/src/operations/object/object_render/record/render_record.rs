use crate::operations::object::{LightRecord, ViewRecord};
use branded_id::IdVec;
use ty_math::TySrgbU8;
use voxrender::{BRenderLight, BRenderView, RenderBloom, RenderOcclusion};

/// A whole render run. Its transforms stay in their configured shapes
/// because their frames need the scene.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderRecord {
    /// The image width in pixels.
    pub width: u32,

    /// The image height in pixels.
    pub height: u32,

    /// The color under the pixels no ray hits, or `None` to leave them
    /// transparent.
    pub background: Option<TySrgbU8>,

    /// The occlusion the render shades with.
    pub occlusion: RenderOcclusion,

    /// One voxel's edge length in meters.
    pub voxel_size: f64,

    /// The halo over the emissive term.
    pub bloom: RenderBloom,

    /// The views by id.
    pub views: IdVec<BRenderView, ViewRecord>,

    /// The lights by id.
    pub lights: IdVec<BRenderLight, LightRecord>,
}
