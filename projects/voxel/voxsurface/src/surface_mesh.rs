use crate::SurfaceSpan;
use ty_math::TyVector3F32;

/// The quads over a grid's faces, triangulated in grid units: a solid cell
/// at `(x, y, z)` fills the unit cube `[x, x+1] x [y, y+1] x [z, z+1]` on
/// the grid's axes. Every quad carries four vertices of its own, each with
/// the face normal, so faces never share vertices and shading stays flat.
/// The vertices run in the order of the span's
/// [`corners`](SurfaceSpan::corners). Triangles wind counter-clockwise seen
/// from outside.
#[derive(Clone, Debug)]
pub struct SurfaceMesh<C> {
    /// One position per vertex, in grid units.
    pub positions: Vec<TyVector3F32>,

    /// One outward face normal per vertex, aligned with
    /// [`positions`](Self::positions).
    pub normals: Vec<TyVector3F32>,

    /// Triangle indices into [`positions`](Self::positions), three per
    /// triangle.
    pub indices: Vec<u32>,

    /// The span each quad covers, one per quad in emission order.
    pub spans: Vec<SurfaceSpan>,

    /// The cells each quad covers, one entry per quad in the order of its
    /// span's [`cells`](SurfaceSpan::cells).
    pub face_cells: Vec<Vec<C>>,
}

impl<C> SurfaceMesh<C> {
    /// Number of quads, two triangles each.
    pub fn quad_count(&self) -> usize {
        self.spans.len()
    }
}

impl<C> Default for SurfaceMesh<C> {
    fn default() -> Self {
        SurfaceMesh {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            spans: Vec::new(),
            face_cells: Vec::new(),
        }
    }
}
