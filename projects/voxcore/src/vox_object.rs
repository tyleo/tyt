use crate::{BVoxPalette, BVoxPaletteCell, BVoxPaletteRef, BVoxVoxel, VoxLiveness};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};
use ty_math::TyVector3U32;

/// One object's voxel volume: a dense grid, the palettes it references, and the
/// cell each voxel samples from each palette.
///
/// Every grid cell has a voxel id equal to its raster index `x*Y*Z + y*Z + z`,
/// so [`voxel_id`](Self::voxel_id) and [`voxel_position`](Self::voxel_position)
/// interconvert. [`is_live`](Self::is_live) says which cells are filled.
///
/// Fields are private because the columns must stay in lockstep with their id
/// pools.
#[derive(Debug, Default)]
pub struct VoxObject {
    /// Display name.
    name: String,

    /// Grid size in voxels.
    bounds: TyVector3U32,

    /// One id per grid cell; id equals the raster index.
    voxel_ids: IdStruct<BVoxVoxel>,

    /// Which cells are filled, one bit per voxel id.
    liveness: VoxLiveness,

    /// Pool shared by `palette_refs` and `samples`.
    palette_ref_ids: IdStruct<BVoxPaletteRef>,

    /// The palette each reference points to, in resolution order.
    palette_refs: IdField<BVoxPaletteRef, U32Id<BVoxPalette>>,

    /// Per reference, the cell each voxel samples (filler in non-live cells).
    samples: IdField<BVoxPaletteRef, IdField<BVoxVoxel, U32Id<BVoxPaletteCell>>>,
}

impl VoxObject {
    /// Largest dense grid an object may allocate, in cells. The grid stores every
    /// cell whether live or not, so this caps memory. Always `<= u32::MAX`, so a
    /// voxel id is always a valid raster index.
    pub const MAX_GRID_CELLS: u64 = 1 << 24;

    /// Creates an empty grid of size `bounds`: every cell has a voxel id, none is
    /// live, and no palettes are referenced yet. Then use
    /// [`add_palette_ref`](Self::add_palette_ref) and
    /// [`retain_voxel`](Self::retain_voxel). `None` if the grid would exceed
    /// [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS).
    pub fn new(name: String, bounds: TyVector3U32) -> Option<Self> {
        let volume = Self::volume_of(bounds);
        if volume > Self::MAX_GRID_CELLS {
            return None;
        }
        let mut voxel_ids = IdStruct::new();
        for _ in 0..volume {
            voxel_ids.retain();
        }
        Some(Self {
            name,
            bounds,
            voxel_ids,
            liveness: VoxLiveness::new(volume as usize),
            palette_ref_ids: IdStruct::new(),
            palette_refs: IdField::new(),
            samples: IdField::new(),
        })
    }

    /// Cell count `X*Y*Z`, saturating (oversized bounds are rejected by
    /// [`new`](Self::new)).
    fn volume_of(bounds: TyVector3U32) -> u64 {
        (bounds.x as u64)
            .saturating_mul(bounds.y as u64)
            .saturating_mul(bounds.z as u64)
    }

    /// Display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Grid size in voxels.
    pub fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    /// Number of live (filled) voxels.
    pub fn live_count(&self) -> usize {
        self.liveness.count_live()
    }

    /// Voxel id at grid `position`, or `None` if outside [`bounds`](Self::bounds).
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

    /// Grid position of `id`, or `None` if outside the grid. Inverse of
    /// [`voxel_id`](Self::voxel_id).
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

    /// Whether the voxel at `id` is live. `false` if outside the grid.
    pub fn is_live(&self, id: U32Id<BVoxVoxel>) -> bool {
        (id.to_u32() as usize) < self.liveness.len() && self.liveness.is_live(id)
    }

    /// Cell the live voxel `id` takes from `palette_ref`, or `None` if the voxel
    /// is not live or `palette_ref` is not one of this object's references.
    pub fn voxel_cell(
        &self,
        id: U32Id<BVoxVoxel>,
        palette_ref: U32Id<BVoxPaletteRef>,
    ) -> Option<U32Id<BVoxPaletteCell>> {
        if !self.is_live(id) || !self.palette_ref_ids.is_retained(palette_ref) {
            return None;
        }
        // Safety: the dense grid gives every reference a slot for every voxel id.
        let column = unsafe { self.samples.get(palette_ref) };
        Some(*unsafe { column.get(id) })
    }

    /// Live voxel ids in ascending raster order. Recover positions with
    /// [`voxel_position`](Self::voxel_position) and cells with
    /// [`voxel_cell`](Self::voxel_cell).
    pub fn iter_live(&self) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
        self.liveness.iter_live()
    }

    /// Number of palette references.
    pub fn palette_ref_count(&self) -> usize {
        self.palette_ref_ids.len()
    }

    /// Palette references in resolution order, as `(reference id, palette)`. Pair
    /// a reference id with [`voxel_cell`](Self::voxel_cell) to read its samples.
    pub fn iter_palette_refs(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>)> + '_ {
        // Safety: retained reference ids have a `palette_refs` value.
        self.palette_ref_ids
            .iter()
            .map(move |id| (id, *unsafe { self.palette_refs.get(id) }))
    }

    /// Adds a palette reference after any existing ones and returns its id,
    /// back-filling every voxel with `default_sample`. Live voxels keep
    /// `default_sample` until [`retain_voxel`](Self::retain_voxel) overwrites it,
    /// so widening the reference set never requires re-adding voxels.
    pub fn add_palette_ref(
        &mut self,
        palette: U32Id<BVoxPalette>,
        default_sample: U32Id<BVoxPaletteCell>,
    ) -> U32Id<BVoxPaletteRef> {
        let palette_ref_id = self.palette_ref_ids.retain();
        self.palette_refs.retain(palette_ref_id, palette);

        let mut column = IdField::new();
        for voxel_id in &self.voxel_ids {
            column.retain(voxel_id, default_sample);
        }
        self.samples.retain(palette_ref_id, column);

        palette_ref_id
    }

    /// Makes the voxel at `id` live with one `samples` cell per palette reference,
    /// in [`add_palette_ref`](Self::add_palette_ref) order. `None`, changing
    /// nothing, if `id` is outside the grid or `samples` has the wrong length.
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
            // Safety: the dense grid gives every reference a slot for `id`.
            let column = unsafe { self.samples.get_mut(palette_ref_id) };
            unsafe { column.set(id, cell) };
        }
        Some(())
    }

    /// Makes the voxel at `id` empty, leaving its samples in place but ignored.
    /// `None`, changing nothing, if `id` is outside the grid.
    pub fn release_voxel(&mut self, id: U32Id<BVoxVoxel>) -> Option<()> {
        if (id.to_u32() as usize) >= self.liveness.len() {
            return None;
        }
        self.liveness.set_live(id, false);
        Some(())
    }

    /// Deep copy. Liveness lives in the id pools, so the columns can't derive
    /// `Clone`; rebuild them against the cloned pools.
    pub fn clone_object(&self) -> Self {
        // The inner sample columns own storage, so clone them one by one;
        // `palette_refs` is Copy-valued and clones wholesale below.
        let mut samples = IdField::new();
        for palette_ref_id in self.palette_ref_ids.iter() {
            // Safety: retained reference ids have a sample column.
            let column = unsafe { self.samples.get(palette_ref_id) };
            samples.retain(palette_ref_id, column.clone());
        }
        Self {
            name: self.name.clone(),
            bounds: self.bounds,
            voxel_ids: self.voxel_ids.clone(),
            liveness: self.liveness.clone(),
            palette_ref_ids: self.palette_ref_ids.clone(),
            palette_refs: self.palette_refs.clone(),
            samples,
        }
    }
}

impl Drop for VoxObject {
    fn drop(&mut self) {
        // Safety: every `palette_ref_ids` id has a value in both columns. Sample
        // cells are Copy, so dropping the inner columns frees their storage and
        // the voxel pool needs no part here.
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
        let object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 3, 4)).unwrap();
        let position = TyVector3U32::new(1, 2, 3);
        let id = object.voxel_id(position).unwrap();
        assert_eq!(id.to_u32(), 23); // 1*(3*4) + 2*4 + 3
        assert_eq!(object.voxel_position(id), Some(position));
        // Out of bounds yields None rather than erroring.
        assert_eq!(object.voxel_id(TyVector3U32::new(2, 0, 0)), None);
        assert_eq!(object.voxel_position(U32Id::from_u32(24)), None);
    }

    // The at-cap case allocates 256^3 voxel ids, which Miri interprets far too
    // slowly; the cap arithmetic exercises no unsafe, so skip it under Miri.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn new_rejects_grid_past_the_cell_cap() {
        // 2048^3 = 2^33 cells, well past MAX_GRID_CELLS (2^24).
        assert!(VoxObject::new("huge".to_owned(), TyVector3U32::new(2048, 2048, 2048)).is_none());
        assert!(VoxObject::new("max".to_owned(), TyVector3U32::new(256, 256, 256)).is_some());
    }

    #[test]
    fn retain_and_release_track_liveness_and_samples() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
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
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_palette_ref(U32Id::<BVoxPalette>::from_u32(0), cell(0));
        let id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

        // Wrong sample arity and an out-of-grid id are both rejected, untouched.
        assert_eq!(object.retain_voxel(id, &[]), None);
        assert_eq!(object.retain_voxel(U32Id::from_u32(99), &[cell(0)]), None);
        assert_eq!(object.live_count(), 0);
        assert_eq!(object.release_voxel(U32Id::from_u32(99)), None);
    }

    #[test]
    fn iter_live_decodes_positions_in_raster_order() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 3, 4)).unwrap();
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

    #[test]
    fn clone_object_is_an_independent_deep_copy() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let palette_ref = object.add_palette_ref(U32Id::<BVoxPalette>::from_u32(3), cell(0));
        let id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(id, &[cell(7)]).unwrap();

        let copy = object.clone_object();
        assert_eq!(copy.name(), "o");
        assert_eq!(copy.bounds(), TyVector3U32::new(2, 1, 1));
        assert_eq!(copy.live_count(), 1);
        assert_eq!(copy.voxel_cell(id, palette_ref), Some(cell(7)));
        assert_eq!(
            copy.iter_palette_refs().collect::<Vec<_>>(),
            [(palette_ref, U32Id::<BVoxPalette>::from_u32(3))]
        );

        // Editing the original must not touch the copy.
        object.release_voxel(id).unwrap();
        assert_eq!(object.live_count(), 0);
        assert_eq!(copy.live_count(), 1);
    }
}
