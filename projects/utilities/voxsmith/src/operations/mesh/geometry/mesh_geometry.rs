use branded_id::U32Id;
use ty_math::TyVector3F32;
use voxcore::BVoxVoxel;

/// Triangulated voxel geometry in voxel-grid space: a live voxel at grid
/// position `(x, y, z)` fills the unit cube `[x, x+1] x [y, y+1] x [z, z+1]`,
/// Z-up as the voxel-json format is. A glTF writer applies the real-world scale
/// and the Z-up-to-Y-up conversion; the mesher itself stays in grid units.
///
/// Every quad carries four of its own vertices with the face normal, so faces
/// never share vertices and shading stays flat. Triangles wind
/// counter-clockwise seen from outside, glTF's front face.
#[derive(Clone, Debug, Default)]
pub(crate) struct MeshGeometry {
    /// One position per vertex, in voxel-grid units.
    pub positions: Vec<TyVector3F32>,

    /// One outward face normal per vertex, aligned with
    /// [`positions`](Self::positions).
    pub normals: Vec<TyVector3F32>,

    /// Triangle indices into [`positions`](Self::positions), three per triangle.
    pub indices: Vec<u32>,

    /// One material index per vertex, aligned with
    /// [`positions`](Self::positions), when meshed with a material key; empty
    /// for the pure-geometry mesh. Every vertex of a quad shares one index,
    /// since material-keyed meshing merges only same-material faces.
    pub material_indices: Vec<u32>,

    /// The voxels each quad covers, one entry per quad in emission order.
    pub face_voxel_ids: Vec<Vec<U32Id<BVoxVoxel>>>,
}

impl MeshGeometry {
    /// Number of quads, two triangles each.
    pub(crate) fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }
}
