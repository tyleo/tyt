use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::BVoxVoxel;
use voxrender::RenderObject;
use voxsurface::SurfaceGrid;

/// An object's opaque cells, each read with its material.
pub struct MaterialGrid<'a> {
    pub object: &'a RenderObject,

    /// Whether a material id passes no light.
    pub opaque: &'a dyn Fn(u32) -> bool,
}

impl SurfaceGrid for MaterialGrid<'_> {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.object.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.object.voxel_id(position).filter(|&voxel_id| {
            self.object
                .voxel_material(voxel_id)
                .is_some_and(|material| (self.opaque)(material.to_u32()))
        })
    }

    fn is_opaque(&self, _cell: Self::Cell) -> bool {
        true
    }

    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool {
        self.object.voxel_material(cell) == self.object.voxel_material(other)
    }
}
