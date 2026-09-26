use crate::{
    Error, Result,
    operations::voxelize::{
        DecodedImage, MeshTriangle, PlacedObject, PlacedPrimitive, TextureSlots, triangle_bounds,
    },
};
use branded_id::U32Id;
use meshdoc::{BMeshImage, MeshState};
use std::collections::HashMap;
use ty_math::TyBoundsF64;

/// The triangle mesh of every placed object in world space, flattened from a
/// mesh document with every node transform applied. The document's axes are
/// the grid's, so a mesh `+y` is a voxel `+y`. Materials and textures are
/// read from the document. Only the images the sampled slots draw are
/// decoded.
pub(crate) struct MeshInput<'a> {
    /// The placed objects, in hierarchy walk order.
    pub objects: Vec<PlacedObject>,

    /// The placed primitives, indexed by a triangle's tag.
    pub primitives: Vec<PlacedPrimitive<'a>>,

    /// The triangles to rasterize, each tagged with its placed primitive,
    /// grouped by placed object.
    pub triangles: Vec<MeshTriangle>,

    /// The decoded images the sampled slots draw, by image id.
    pub images: HashMap<U32Id<BMeshImage>, DecodedImage>,

    /// The document state the materials and textures are read from.
    pub state: &'a MeshState,
}

impl MeshInput<'_> {
    /// Whether any placed primitive samples a texture, the case `auto`
    /// samples per texel.
    pub fn is_textured(&self) -> bool {
        (0..self.primitives.len()).any(|index| TextureSlots::resolve(self, index as u32).any())
    }

    /// The triangles of `object`.
    pub fn object_triangles(&self, object: &PlacedObject) -> &[MeshTriangle] {
        &self.triangles[object.range.clone()]
    }

    /// The bounding box of `object` in world space. Errors when it has no
    /// triangles, reporting the name it takes under `fallback_name`.
    pub fn object_bounds(
        &self,
        object: &PlacedObject,
        fallback_name: Option<&str>,
    ) -> Result<TyBoundsF64> {
        triangle_bounds(self.object_triangles(object)).ok_or_else(|| {
            let message = match object.name(fallback_name) {
                "" => "an unnamed object has no triangle geometry".to_owned(),
                name => format!("object `{name}` has no triangle geometry"),
            };
            Error::invalid(message)
        })
    }
}
