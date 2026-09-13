use crate::operations::voxelize::{MeshMaterial, MeshMaterialMaps, MeshTexture, MeshTriangle};
use ty_math::{TyBoundsF64, TyVector3F64};

/// A triangle mesh in world space on voxcore's Z-up axes, flattened from a
/// mesh document with every node transform applied, the one shape
/// [`voxelize_mesh`] rasterizes.
///
/// [`voxelize_mesh`]: crate::operations::voxelize::voxelize_mesh
pub struct MeshInput {
    /// The triangles to rasterize, each tagged with its material.
    pub triangles: Vec<MeshTriangle>,

    /// The distinct materials the triangles reference, indexed by a
    /// triangle's material tag.
    pub materials: Vec<MeshMaterial>,

    /// Each material's optional texture bindings, parallel to
    /// [`materials`](Self::materials).
    pub maps: Vec<MeshMaterialMaps>,

    /// The decoded texture images a map binding indexes.
    pub textures: Vec<MeshTexture>,

    /// The mesh's name, for the voxelized object. `None` when the source has
    /// none.
    pub name: Option<String>,
}

impl MeshInput {
    /// Whether any material carries a texture map, the case `auto` samples
    /// per texel.
    pub fn is_textured(&self) -> bool {
        self.maps.iter().any(MeshMaterialMaps::any)
    }

    /// The size of the mesh's bounding box in meters on the Z-up axes, which
    /// a caller divides to choose a grid resolution. Zero on every axis when
    /// the mesh has no triangles.
    pub fn extent(&self) -> TyVector3F64 {
        let points = self.triangles.iter().flat_map(|triangle| triangle.points);

        match TyBoundsF64::from_points(points) {
            Some(bounds) => bounds.size(),
            None => TyVector3F64::ZERO,
        }
    }
}
