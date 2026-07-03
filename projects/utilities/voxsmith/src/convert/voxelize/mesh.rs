use crate::{MeshMaterial, MeshMaterialMaps, MeshTexture, MeshTriangle, triangle_bounds};
use ty_math::TyVector3F64;

/// A triangle mesh in world space on the Voxel Json Z-up axes: the geometry to
/// voxelize plus the materials, base-color textures, and texture coordinates its
/// triangles reference. Every mesh-format reader converts its own axes to this
/// Z-up convention, so [`voxelize_mesh`] and the readers meet on one
/// format-independent type.
///
/// [`voxelize_mesh`]: crate::voxelize_mesh
pub struct Mesh {
    /// The triangles to rasterize, each tagged with its material.
    pub(crate) triangles: Vec<MeshTriangle>,

    /// The distinct materials the triangles reference, indexed by a triangle's
    /// material tag.
    pub(crate) materials: Vec<MeshMaterial>,

    /// Each material's optional texture bindings, parallel to
    /// [`materials`](Self::materials).
    pub(crate) maps: Vec<MeshMaterialMaps>,

    /// The decoded texture images a map binding indexes.
    pub(crate) textures: Vec<MeshTexture>,

    /// The mesh's name, for the voxelized object; `None` when the source names
    /// it nothing.
    pub(crate) name: Option<String>,
}

impl Mesh {
    /// Whether any material carries a texture map, so per-texel sampling has
    /// something to read and `auto` picks it.
    pub(crate) fn is_textured(&self) -> bool {
        self.maps.iter().any(MeshMaterialMaps::any)
    }

    /// The real-world size of the mesh's bounding box, in meters, on the Z-up
    /// axes. This is the figure a caller divides to choose a grid resolution.
    /// Zero on every axis when the mesh has no triangles.
    pub fn extent(&self) -> TyVector3F64 {
        let points = self.triangles.iter().flat_map(|triangle| triangle.points);

        match triangle_bounds(points) {
            Some((min, max)) => max - min,
            None => TyVector3F64::default(),
        }
    }
}
