use crate::operations::voxelize::MeshTriangle;
use ty_math::TyBoundsF64;

/// The axis-aligned bounding box of a triangle soup, or `None` when it is
/// empty.
pub(crate) fn triangle_bounds(triangles: &[MeshTriangle]) -> Option<TyBoundsF64> {
    TyBoundsF64::from_points(triangles.iter().flat_map(|triangle| triangle.points))
}
