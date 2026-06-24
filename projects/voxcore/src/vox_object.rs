use crate::{BVoxPalette, BVoxPaletteCell, BVoxPaletteRef, BVoxVoxel, VoxLiveness};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};
use ty_math::TyVector3U32;

/// One object's voxel volume: a dense voxel grid, the palettes it references,
/// and a sample per voxel per palette reference.
///
/// An object is pure geometry placed by a
/// [`VoxHierarchyNode`](crate::VoxHierarchyNode) that references it. The grid
/// spans [`bounds`](Self::bounds) and allocates a voxel id for every cell, with
/// id equal to the cell's raster index `x * Y * Z + y * Z + z`, so a voxel id
/// and its grid position convert directly. Which cells are filled is held by
/// [`liveness`](Self::liveness); a sample is always a valid cell reference and
/// so cannot by itself mark a cell empty.
///
/// [`palette_refs`](Self::palette_refs) lists the shared
/// [`VoxPalette`](crate::VoxPalette)s it samples, in resolution order;
/// [`samples`](Self::samples) carries, per palette reference, the cell each
/// voxel takes from that palette. Cells outside [`liveness`](Self::liveness)
/// carry an unused filler so every column stays in sync with
/// [`voxel_ids`](Self::voxel_ids).
///
/// The columns track liveness through the id pools, so this type does not
/// derive `Clone` or `PartialEq`; its [`Drop`] releases every column.
#[derive(Debug, Default)]
pub struct VoxObject {
    /// The display name of the object.
    pub name: String,

    /// The `[x, y, z]` size in voxels of the dense voxel grid.
    pub bounds: TyVector3U32,

    /// The voxel id pool: one retained id per grid cell, with the id equal to
    /// the cell's raster index `x * Y * Z + y * Z + z`.
    pub voxel_ids: IdStruct<BVoxVoxel>,

    /// Which grid cells are filled, one bit per voxel id.
    pub liveness: VoxLiveness,

    /// The palette reference id pool, shared by
    /// [`palette_refs`](Self::palette_refs) and [`samples`](Self::samples).
    pub palette_ref_ids: IdStruct<BVoxPaletteRef>,

    /// The shared palette each reference points to, in resolution order.
    pub palette_refs: IdField<BVoxPaletteRef, U32Id<BVoxPalette>>,

    /// One column per palette reference: each maps a voxel to the cell it takes
    /// from that reference's palette. Cells outside [`liveness`](Self::liveness)
    /// hold an unused filler.
    pub samples: IdField<BVoxPaletteRef, IdField<BVoxVoxel, U32Id<BVoxPaletteCell>>>,
}

impl VoxObject {
    /// Decodes a voxel id into its `[x, y, z]` grid position, the inverse of the
    /// raster index `x * Y * Z + y * Z + z` that defines the id.
    ///
    /// # Panics
    /// Panics if [`bounds`](Self::bounds) has a zero `y` or `z` extent, which no
    /// in-bounds voxel can have.
    pub fn voxel_position(&self, id: U32Id<BVoxVoxel>) -> TyVector3U32 {
        let raster = id.to_u32();
        let plane = self.bounds.y * self.bounds.z;
        TyVector3U32::new(
            raster / plane,
            (raster % plane) / self.bounds.z,
            raster % self.bounds.z,
        )
    }

    /// Iterates the ids of the object's live voxels in ascending raster order,
    /// for quick traversal of just the filled cells (a scan over the set bits,
    /// touching nothing for empty space). Use
    /// [`voxel_position`](Self::voxel_position) to recover a voxel's grid
    /// position and [`samples`](Self::samples) to read its palette cells.
    pub fn iter_live(&self) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
        self.liveness.iter_live()
    }
}

impl Drop for VoxObject {
    fn drop(&mut self) {
        // Safety: `palette_refs` and `samples` retain a value for every id in
        // `palette_ref_ids`. Releasing `palette_refs` drops its `Copy` palette
        // ids; releasing `samples` drops each inner per-voxel column (freeing
        // that column's storage). Those columns hold only `Copy` cell ids, which
        // have no destructor, so leaking them rather than releasing per voxel is
        // sound and the voxel pool needs no part in this drop.
        unsafe {
            self.palette_refs.release_all(&self.palette_ref_ids);
            self.samples.release_all(&self.palette_ref_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{VoxLiveness, VoxObject};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    #[test]
    fn iter_live_decodes_positions_in_raster_order() {
        let mut object = VoxObject::default();
        object.bounds = TyVector3U32::new(2, 3, 4);
        let volume = 2 * 3 * 4;
        for _ in 0..volume {
            object.voxel_ids.retain();
        }
        object.liveness = VoxLiveness::new(volume);

        // (0,0,0) -> 0, (0,1,2) -> 6, (1,2,3) -> 23, set out of order.
        for raster in [23, 0, 6] {
            object.liveness.set_live(U32Id::from_u32(raster), true);
        }

        let live: Vec<(u32, [u32; 3])> = object
            .iter_live()
            .map(|id| {
                let position = object.voxel_position(id);
                (id.to_u32(), [position.x, position.y, position.z])
            })
            .collect();
        assert_eq!(
            live,
            [(0, [0, 0, 0]), (6, [0, 1, 2]), (23, [1, 2, 3])]
        );
    }
}
