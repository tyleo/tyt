use crate::{Error, Result, gltf_triangles, triangle_bounds};
use ty_math::TyVector3;

/// The real-world size of a glTF or GLB mesh's bounding box, in meters, on the
/// Voxel Json Z-up axes. This is the figure a caller divides to choose a grid
/// resolution, so [`voxelize_gltf`](crate::voxelize_gltf) takes plain voxel
/// counts and never re-reads the extent. Errors when the mesh has no triangle
/// geometry.
pub fn gltf_mesh_extent(bytes: &[u8]) -> Result<TyVector3<f64>> {
    let triangles = gltf_triangles(bytes)?;
    let (min, max) = triangle_bounds(&triangles)
        .ok_or_else(|| Error::invalid("glTF has no triangle geometry to voxelize"))?;
    Ok(TyVector3::new(
        max[0] - min[0],
        max[1] - min[1],
        max[2] - min[2],
    ))
}
