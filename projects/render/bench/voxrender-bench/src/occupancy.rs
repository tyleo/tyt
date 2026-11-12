use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::BVoxVoxel;
use voxrender::RenderObject;
use voxsurface::SurfaceGrid;

/// An object's cells read by occupancy alone. Every live cell is opaque, so
/// a greedy run merges across materials.
pub struct Occupancy<'a>(pub &'a RenderObject);

impl SurfaceGrid for Occupancy<'_> {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.0.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.0
            .voxel_id(position)
            .filter(|&voxel_id| self.0.is_live(voxel_id))
    }

    fn is_opaque(&self, _cell: Self::Cell) -> bool {
        true
    }

    fn shares_material(&self, _cell: Self::Cell, _other: Self::Cell) -> bool {
        true
    }
}
