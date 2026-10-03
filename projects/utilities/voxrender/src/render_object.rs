use crate::{BRenderMaterial, Error, Result};
use branded_id::{IdVec, U32Id, ext::IteratorExt};
use ty_math::TyVector3U32;
use voxcore::{BVoxVoxel, VoxObject};
use voxsurface::SurfaceGrid;

/// An object flattened for drawing: a dense grid of cells, each empty or
/// holding one material of the scene. A voxel at `p` fills the unit cube
/// with min corner `p` in grid units. A cell's id is voxcore's voxel id for
/// its position, so a cell shares its id with the voxel it mirrors.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderObject {
    name: String,

    bounds: TyVector3U32,

    voxels: IdVec<BVoxVoxel, Option<U32Id<BRenderMaterial>>>,
}

impl RenderObject {
    /// An empty grid of `bounds` named `name`. Errors if the grid would
    /// exceed [`MAX_GRID_CELLS`](VoxObject::MAX_GRID_CELLS).
    pub fn new(name: String, bounds: TyVector3U32) -> Result<Self> {
        let cells = VoxObject::volume_of(bounds);

        if cells > VoxObject::MAX_GRID_CELLS {
            return Err(Error::GridCellCap { cells });
        }

        Ok(RenderObject {
            name,
            bounds,
            voxels: IdVec::from(vec![None; cells as usize]),
        })
    }

    /// The display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Renames the object.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Grid size in voxels.
    pub fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    /// The id of the cell at `position`, or `None` outside the grid.
    pub fn voxel_id(&self, position: TyVector3U32) -> Option<U32Id<BVoxVoxel>> {
        VoxObject::raster_id(self.bounds, position)
    }

    /// The position of the cell `id`, or `None` outside the grid. Inverse of
    /// [`voxel_id`](Self::voxel_id).
    pub fn voxel_position(&self, id: U32Id<BVoxVoxel>) -> Option<TyVector3U32> {
        VoxObject::raster_position(self.bounds, id)
    }

    /// The material the cell `id` holds, or `None` where the cell is empty
    /// or outside the grid.
    pub fn voxel_material(&self, id: U32Id<BVoxVoxel>) -> Option<U32Id<BRenderMaterial>> {
        self.voxels.get(id.to_usize_id()).copied().flatten()
    }

    /// Whether the cell `id` holds a material.
    pub fn is_live(&self, id: U32Id<BVoxVoxel>) -> bool {
        self.voxel_material(id).is_some()
    }

    /// Writes the material the cell `id` holds, `None` to empty it. Errors,
    /// changing nothing, if `id` is outside the grid. A scene checks the
    /// material when it retains the object.
    pub fn set_voxel_material(
        &mut self,
        id: U32Id<BVoxVoxel>,
        material_id: Option<U32Id<BRenderMaterial>>,
    ) -> Result<()> {
        let Some(cell) = self.voxels.get_mut(id.to_usize_id()) else {
            return Err(Error::UnknownVoxel { voxel_id: id });
        };

        *cell = material_id;

        Ok(())
    }

    /// Every live cell with its material, in raster order.
    pub fn iter_live(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxVoxel>, U32Id<BRenderMaterial>)> + '_ {
        self.voxels
            .iter()
            .enumerate_ids()
            .filter_map(|(voxel_id, material_id)| {
                material_id.map(|material_id| (voxel_id, material_id))
            })
    }

    /// Number of live cells.
    pub fn live_count(&self) -> usize {
        self.voxels.iter().filter(|cell| cell.is_some()).count()
    }

    /// The min and max positions over the live cells, or `None` with none
    /// live.
    pub fn live_extent(&self) -> Option<(TyVector3U32, TyVector3U32)> {
        self.iter_live()
            .map(|(id, _)| {
                self.voxel_position(id)
                    .expect("a live cell is within the grid")
            })
            .fold(None, |extent, position| {
                Some(match extent {
                    None => (position, position),
                    Some((min, max)) => (min.min(position), max.max(position)),
                })
            })
    }
}

/// A cell is solid where it holds a material.
impl SurfaceGrid for RenderObject {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.voxel_id(position).filter(|&id| self.is_live(id))
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, RenderObject};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxsurface::SurfaceGrid;

    #[test]
    fn cells_address_by_raster_id_and_hold_one_material() {
        let mut object = RenderObject::new("o".to_owned(), TyVector3U32::new(2, 3, 4)).unwrap();

        let position = TyVector3U32::new(1, 2, 3);
        let id = object.voxel_id(position).unwrap();

        assert_eq!(id.to_u32(), 12 + 2 * 4 + 3);
        assert_eq!(object.voxel_position(id), Some(position));
        assert_eq!(object.voxel_id(TyVector3U32::new(2, 0, 0)), None);
        assert_eq!(object.voxel_position(U32Id::from_u32(24)), None);

        assert!(!object.is_live(id));
        assert_eq!(object.cell(position), None);
        assert_eq!(object.live_extent(), None);

        object
            .set_voxel_material(id, Some(U32Id::from_u32(7)))
            .unwrap();

        assert_eq!(object.voxel_material(id), Some(U32Id::from_u32(7)));
        assert_eq!(object.cell(position), Some(id));
        assert!(object.is_solid([1, 2, 3]));
        assert_eq!(
            object.iter_live().collect::<Vec<_>>(),
            [(id, U32Id::from_u32(7))]
        );
        assert_eq!(object.live_count(), 1);
        assert_eq!(object.live_extent(), Some((position, position)));

        assert_eq!(
            object.set_voxel_material(U32Id::from_u32(24), None),
            Err(Error::UnknownVoxel {
                voxel_id: U32Id::from_u32(24)
            })
        );
    }

    #[test]
    fn a_grid_past_the_cell_cap_is_refused() {
        assert_eq!(
            RenderObject::new("o".to_owned(), TyVector3U32::new(1 << 10, 1 << 10, 1 << 8)),
            Err(Error::GridCellCap { cells: 1 << 28 })
        );
    }
}
