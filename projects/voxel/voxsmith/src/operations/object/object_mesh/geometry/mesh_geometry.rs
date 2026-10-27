use branded_id::U32Id;
use voxcore::BVoxVoxel;
use voxsurface::SurfaceMesh;

/// A [`SurfaceMesh`] over an object's voxels. Its face cells are voxel ids.
pub type MeshGeometry = SurfaceMesh<U32Id<BVoxVoxel>>;
