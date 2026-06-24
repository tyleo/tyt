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
    /// Creates a dense, fully empty grid of size `bounds`: every cell is
    /// allocated a voxel id (so positions and ids interconvert) but none are
    /// live, and the object references no palettes yet. Add references with
    /// [`add_palette_ref`](Self::add_palette_ref), then fill cells with
    /// [`retain_voxel`](Self::retain_voxel).
    ///
    /// The grid materializes storage for every cell, so `bounds` must be small
    /// enough that its `X * Y * Z` cells fit in memory.
    pub fn new(name: String, bounds: TyVector3U32) -> Self {
        let volume = Self::volume_of(bounds);
        let mut voxel_ids = IdStruct::new();
        for _ in 0..volume {
            voxel_ids.retain();
        }
        Self {
            name,
            bounds,
            voxel_ids,
            liveness: VoxLiveness::new(volume as usize),
            palette_ref_ids: IdStruct::new(),
            palette_refs: IdField::new(),
            samples: IdField::new(),
        }
    }

    /// The number of grid cells, `X * Y * Z`, saturating rather than overflowing
    /// for absurd bounds (which could never be materialized anyway).
    fn volume_of(bounds: TyVector3U32) -> u64 {
        (bounds.x as u64)
            .saturating_mul(bounds.y as u64)
            .saturating_mul(bounds.z as u64)
    }

    /// The number of live (filled) voxels.
    pub fn live_count(&self) -> usize {
        self.liveness.count_live()
    }

    /// The voxel id at grid `position`, or `None` if it lies outside
    /// [`bounds`](Self::bounds). The id is the raster index
    /// `x * Y * Z + y * Z + z`.
    pub fn voxel_id(&self, position: TyVector3U32) -> Option<U32Id<BVoxVoxel>> {
        if position.x >= self.bounds.x || position.y >= self.bounds.y || position.z >= self.bounds.z
        {
            return None;
        }
        let plane = self.bounds.y as u64 * self.bounds.z as u64;
        let raster = (position.x as u64)
            .saturating_mul(plane)
            .saturating_add(position.y as u64 * self.bounds.z as u64)
            .saturating_add(position.z as u64);
        (raster <= u32::MAX as u64).then(|| U32Id::from_u32(raster as u32))
    }

    /// The `[x, y, z]` grid position of `id`, or `None` if it lies outside the
    /// grid. The inverse of [`voxel_id`](Self::voxel_id).
    pub fn voxel_position(&self, id: U32Id<BVoxVoxel>) -> Option<TyVector3U32> {
        let raster = id.to_u32() as u64;
        if raster >= Self::volume_of(self.bounds) {
            return None;
        }
        // A non-zero volume guarantees both extents below are non-zero.
        let plane = self.bounds.y as u64 * self.bounds.z as u64;
        Some(TyVector3U32::new(
            (raster / plane) as u32,
            ((raster % plane) / self.bounds.z as u64) as u32,
            (raster % self.bounds.z as u64) as u32,
        ))
    }

    /// Whether the voxel at `id` is live. `false` for an id outside the grid.
    pub fn is_live(&self, id: U32Id<BVoxVoxel>) -> bool {
        (id.to_u32() as usize) < self.liveness.len() && self.liveness.is_live(id)
    }

    /// The cell the live voxel at `id` takes from the palette reference
    /// `palette_ref`, or `None` if the voxel is not live or `palette_ref` is not
    /// one of this object's references.
    pub fn voxel_cell(
        &self,
        id: U32Id<BVoxVoxel>,
        palette_ref: U32Id<BVoxPaletteRef>,
    ) -> Option<U32Id<BVoxPaletteCell>> {
        if !self.is_live(id) || !self.palette_ref_ids.is_retained(palette_ref) {
            return None;
        }
        // Safety: every palette reference has a sample column holding a value for
        // every voxel id, since the grid is dense.
        let column = unsafe { self.samples.get(palette_ref) };
        Some(*unsafe { column.get(id) })
    }

    /// Iterates the ids of the object's live voxels in ascending raster order,
    /// for quick traversal of just the filled cells (a scan over the set bits,
    /// touching nothing for empty space). Use
    /// [`voxel_position`](Self::voxel_position) to recover a voxel's grid
    /// position and [`voxel_cell`](Self::voxel_cell) to read its palette cells.
    pub fn iter_live(&self) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
        self.liveness.iter_live()
    }

    /// Adds a reference to a shared palette, in resolution order after any
    /// existing references, and returns its id. Allocates the reference's sample
    /// column, in which every voxel takes `placeholder` until
    /// [`retain_voxel`](Self::retain_voxel) sets its cell. The placeholder only
    /// occupies cells that are not live and is never read back through them, so
    /// any cell of `palette` serves.
    pub fn add_palette_ref(
        &mut self,
        palette: U32Id<BVoxPalette>,
        placeholder: U32Id<BVoxPaletteCell>,
    ) -> U32Id<BVoxPaletteRef> {
        let palette_ref_id = self.palette_ref_ids.retain();
        self.palette_refs.retain(palette_ref_id, palette);

        let mut column = IdField::new();
        for voxel_id in &self.voxel_ids {
            column.retain(voxel_id, placeholder);
        }
        self.samples.retain(palette_ref_id, column);

        palette_ref_id
    }

    /// Makes the voxel at `id` live and sets the cell it takes from each palette
    /// reference, in [`add_palette_ref`](Self::add_palette_ref) order. Returns
    /// `None`, changing nothing, if `id` is outside the grid or `samples` does
    /// not carry exactly one cell per palette reference.
    pub fn retain_voxel(
        &mut self,
        id: U32Id<BVoxVoxel>,
        samples: &[U32Id<BVoxPaletteCell>],
    ) -> Option<()> {
        if (id.to_u32() as usize) >= self.liveness.len()
            || samples.len() != self.palette_ref_ids.len()
        {
            return None;
        }

        self.liveness.set_live(id, true);
        for (palette_ref_id, &cell) in self.palette_ref_ids.iter().zip(samples) {
            // Safety: every palette reference's sample column holds a value for
            // every voxel id (the grid is dense), so the slot is initialized.
            let column = unsafe { self.samples.get_mut(palette_ref_id) };
            unsafe { column.set(id, cell) };
        }
        Some(())
    }

    /// Makes the voxel at `id` empty. Returns `None`, changing nothing, if `id`
    /// is outside the grid. The voxel's samples are left in place but ignored
    /// until it is live again.
    pub fn release_voxel(&mut self, id: U32Id<BVoxVoxel>) -> Option<()> {
        if (id.to_u32() as usize) >= self.liveness.len() {
            return None;
        }
        self.liveness.set_live(id, false);
        Some(())
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
    use crate::{BVoxPalette, BVoxPaletteCell, VoxObject};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    fn cell(index: u32) -> U32Id<BVoxPaletteCell> {
        U32Id::from_u32(index)
    }

    #[test]
    fn voxel_id_and_position_round_trip_and_bound_check() {
        let object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 3, 4));
        let position = TyVector3U32::new(1, 2, 3);
        let id = object.voxel_id(position).unwrap();
        assert_eq!(id.to_u32(), 23); // 1*(3*4) + 2*4 + 3
        assert_eq!(object.voxel_position(id), Some(position));
        // Out of bounds in either direction yields None rather than erroring.
        assert_eq!(object.voxel_id(TyVector3U32::new(2, 0, 0)), None);
        assert_eq!(object.voxel_position(U32Id::from_u32(24)), None);
    }

    #[test]
    fn retain_and_release_track_liveness_and_samples() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1));
        let palette_ref = object.add_palette_ref(U32Id::<BVoxPalette>::from_u32(0), cell(0));
        let id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();

        assert!(!object.is_live(id));
        assert_eq!(object.voxel_cell(id, palette_ref), None);

        assert_eq!(object.retain_voxel(id, &[cell(7)]), Some(()));
        assert!(object.is_live(id));
        assert_eq!(object.live_count(), 1);
        assert_eq!(object.voxel_cell(id, palette_ref), Some(cell(7)));
        assert_eq!(object.iter_live().collect::<Vec<_>>(), [id]);

        assert_eq!(object.release_voxel(id), Some(()));
        assert!(!object.is_live(id));
        assert_eq!(object.live_count(), 0);
        assert_eq!(object.voxel_cell(id, palette_ref), None);
    }

    #[test]
    fn retain_voxel_rejects_bad_input_without_changing_state() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1));
        object.add_palette_ref(U32Id::<BVoxPalette>::from_u32(0), cell(0));
        let id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

        // Wrong sample arity (0 cells for 1 reference) and an out-of-grid id are
        // both rejected, leaving the object untouched.
        assert_eq!(object.retain_voxel(id, &[]), None);
        assert_eq!(object.retain_voxel(U32Id::from_u32(99), &[cell(0)]), None);
        assert_eq!(object.live_count(), 0);
        assert_eq!(object.release_voxel(U32Id::from_u32(99)), None);
    }

    #[test]
    fn iter_live_decodes_positions_in_raster_order() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 3, 4));
        // Retain out of order; iteration must still ascend by raster index.
        for position in [
            TyVector3U32::new(1, 2, 3),
            TyVector3U32::new(0, 0, 0),
            TyVector3U32::new(0, 1, 2),
        ] {
            let id = object.voxel_id(position).unwrap();
            object.retain_voxel(id, &[]).unwrap();
        }

        let live: Vec<(u32, [u32; 3])> = object
            .iter_live()
            .map(|id| {
                let position = object.voxel_position(id).unwrap();
                (id.to_u32(), [position.x, position.y, position.z])
            })
            .collect();
        assert_eq!(live, [(0, [0, 0, 0]), (6, [0, 1, 2]), (23, [1, 2, 3])]);
    }
}
