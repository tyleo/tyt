use ty_math::TyVector3F32;

/// Triangulated voxel geometry in voxel-grid space: a live voxel at grid
/// position `(x, y, z)` fills the unit cube `[x, x+1] x [y, y+1] x [z, z+1]`,
/// Z-up as the voxel-json format is. A glTF writer applies the real-world scale
/// and the Z-up-to-Y-up conversion; the mesher itself stays in grid units.
///
/// Every quad carries four of its own vertices with the face normal, so faces
/// never share vertices and shading stays flat. Triangles wind
/// counter-clockwise seen from outside, glTF's front face.
#[derive(Clone, Debug, Default)]
pub struct MeshGeometry {
    /// One position per vertex, in voxel-grid units.
    pub positions: Vec<TyVector3F32>,

    /// One outward face normal per vertex, aligned with
    /// [`positions`](Self::positions).
    pub normals: Vec<TyVector3F32>,

    /// Triangle indices into [`positions`](Self::positions), three per triangle.
    pub indices: Vec<u32>,
}

impl MeshGeometry {
    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Number of quads, two triangles each.
    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }
}
