use crate::operations::voxelize::{
    DecodedImage, MeshTriangle, PlacedPrimitive, TextureSlots, triangle_bounds,
};
use branded_id::U32Id;
use meshdoc::{BMeshImage, MeshState};
use std::collections::HashMap;
use ty_math::TyBoundsF64;

/// A triangle mesh in world space, flattened from a mesh document with every
/// node transform applied, the one shape [`voxelize_mesh`] rasterizes. The
/// document's axes are the grid's, so a mesh `+y` is a voxel `+y`. Materials
/// and textures are read from the document. Only the images the sampled
/// slots draw are decoded.
///
/// [`voxelize_mesh`]: crate::operations::voxelize::voxelize_mesh
pub(crate) struct MeshInput<'a> {
    /// The placed primitives, indexed by a triangle's tag.
    pub primitives: Vec<PlacedPrimitive<'a>>,

    /// The triangles to rasterize, each tagged with its placed primitive.
    pub triangles: Vec<MeshTriangle>,

    /// The decoded images the sampled slots draw, by image id.
    pub images: HashMap<U32Id<BMeshImage>, DecodedImage>,

    /// The document state the materials and textures are read from.
    pub state: &'a MeshState,

    /// The mesh's name, for the voxelized object. `None` when the source has
    /// none.
    pub name: Option<String>,
}

impl MeshInput<'_> {
    /// Whether any placed primitive samples a texture, the case `auto`
    /// samples per texel.
    pub fn is_textured(&self) -> bool {
        (0..self.primitives.len()).any(|index| TextureSlots::resolve(self, index as u32).any())
    }

    /// The mesh's bounding box in world space, or `None` when the mesh has
    /// no triangles.
    pub fn bounds(&self) -> Option<TyBoundsF64> {
        triangle_bounds(&self.triangles)
    }
}
