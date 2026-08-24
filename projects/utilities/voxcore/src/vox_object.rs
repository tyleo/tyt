use crate::{BVoxLayer, BVoxMaterial, BVoxPalette, BVoxVoxel, Error, Result, VoxLiveness};
use branded_id::{
    IdVec, U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use std::collections::HashMap;
use ty_math::{TyVector3I32, TyVector3U32};

/// One object's voxel volume: a dense grid, the ordered layers it references,
/// and the material each voxel samples in each layer.
///
/// Each layer references a shared [`VoxPalette`](crate::VoxPalette), and
/// layers override back to front: each property takes its value from the
/// last layer that supplies it.
///
/// Every grid cell has a voxel id equal to
/// its raster index `x*Y*Z + y*Z + z`, so [`voxel_id`](Self::voxel_id) and
/// [`voxel_position`](Self::voxel_position) interconvert.
/// [`is_live`](Self::is_live) says which cells are filled.
#[derive(Debug, Default)]
pub struct VoxObject {
    /// Display name.
    name: String,

    /// Grid size in voxels: the object's build volume (the author's edit grid).
    /// Live voxels sit anywhere inside `[0, bounds)` and need not fill it; the
    /// tight runtime extent is derived on demand by
    /// [`live_extent`](Self::live_extent). An empty build volume is
    /// `[0, 0, 0]`.
    bounds: TyVector3U32,

    /// Translation from the placing hierarchy node to the build volume's min
    /// corner, in voxels.
    origin: TyVector3I32,

    /// Which cells are filled, one bit per voxel id.
    liveness: VoxLiveness,

    /// Layer id pool shared by `layer_palette_ids` and `samples`.
    layer_ids: IdStruct<BVoxLayer>,

    /// The palette each layer references, in layer order.
    layer_palette_ids: IdField<BVoxLayer, U32Id<BVoxPalette>>,

    /// Per layer, the material each voxel samples, one slot per grid cell.
    /// Cells of non-live voxels are ignored filler.
    samples: IdField<BVoxLayer, IdVec<BVoxVoxel, U32Id<BVoxMaterial>>>,
}

impl VoxObject {
    /// Largest dense grid an object may allocate, in cells. The grid stores
    /// every cell whether live or not, so this caps memory. Because the cap is
    /// `<= u32::MAX`, a voxel id is always a valid raster index.
    pub const MAX_GRID_CELLS: u64 = 1 << 27;

    /// Creates an empty grid of size `bounds`: every cell has a voxel id, none
    /// is live, and no layers are referenced yet. Then use
    /// [`retain_layer`](Self::retain_layer) and [`retain_voxel`](Self::retain_voxel).
    /// Errors if the grid would exceed
    /// [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS).
    pub fn new(name: String, bounds: TyVector3U32) -> Result<Self> {
        let volume = Self::volume_of(bounds);
        if volume > Self::MAX_GRID_CELLS {
            return Err(Error::GridCellCap { cells: volume });
        }

        Ok(Self {
            name,
            bounds,
            origin: TyVector3I32::default(),
            liveness: VoxLiveness::new(volume as usize),
            layer_ids: IdStruct::new(),
            layer_palette_ids: IdField::new(),
            samples: IdField::new(),
        })
    }

    /// Cell count `X*Y*Z`, saturating.
    fn volume_of(bounds: TyVector3U32) -> u64 {
        (bounds.x as u64)
            .saturating_mul(bounds.y as u64)
            .saturating_mul(bounds.z as u64)
    }

    /// Deep copy. Liveness lives in the layer id pool, so the columns can't
    /// derive `Clone`; rebuild them against the cloned pool.
    pub fn clone_object(&self) -> Self {
        // The inner sample columns own storage, so clone them one by one;
        // `layer_palette_ids` is Copy-valued and clones wholesale below.
        let mut samples = IdField::new();
        for layer_id in self.layer_ids.iter() {
            // Safety: retained layer ids have a sample column.
            let column = unsafe { self.samples.get(layer_id) };
            samples.retain(layer_id, column.clone());
        }

        Self {
            name: self.name.clone(),
            bounds: self.bounds,
            origin: self.origin,
            liveness: self.liveness.clone(),
            layer_ids: self.layer_ids.clone(),
            layer_palette_ids: self.layer_palette_ids.clone(),
            samples,
        }
    }

    /// Rewrites this object's cross-references to match id pools a
    /// [`VoxMain`](crate::VoxMain) is compacting, then compacts its own layer
    /// id pool. Each layer's palette is translated through `palette_remap`, and
    /// each layer's live-voxel sample materials through the `material_remaps`
    /// entry for the referenced palette's pre-gc id. Requires a referentially
    /// valid object, so every translation resolves.
    pub(crate) fn gc(
        &mut self,
        palette_remap: &IdRemap<BVoxPalette, u32>,
        material_remaps: &IdVec<BVoxPalette, IdRemap<BVoxMaterial, u32>>,
    ) {
        let layer_ids: Vec<_> = self.layer_ids.iter().collect();
        let live_ids: Vec<_> = self.liveness.iter_live().collect();
        for layer_id in layer_ids {
            // Translate the referenced palette id to its relabeled value.
            // Safety: retained layer ids have a `layer_palette_ids` value.
            let old_palette_id = *unsafe { self.layer_palette_ids.get(layer_id) };
            let new_palette_id = palette_remap
                .new_id(old_palette_id)
                .expect("a layer references a live palette in a valid state");
            // Safety: same retained layer id.
            *unsafe { self.layer_palette_ids.get_mut(layer_id) } = new_palette_id;

            // Translate each live voxel's sample material through that
            // palette's relabeling; non-live voxels' filler cells are exempt.
            let material_remap = &material_remaps[old_palette_id.to_usize_id()];
            // Safety: retained layer ids have a sample column.
            let column = unsafe { self.samples.get_mut(layer_id) };
            for &voxel_id in &live_ids {
                let new_material_id = material_remap
                    .new_id(column[voxel_id.to_usize_id()])
                    .expect("a live voxel samples a live material in a valid state");
                column[voxel_id.to_usize_id()] = new_material_id;
            }
        }

        // Compact the layer id pool; the values above were already translated,
        // so this only relabels layer keys.
        let layer_remap = self.layer_ids.gc();
        // Safety: both columns were in sync with the pre-gc layer id pool, and
        // nothing has retained or released since.
        unsafe { self.samples.gc(&layer_remap) };
        unsafe { self.layer_palette_ids.gc(&layer_remap) };
    }

    /// Grid size in voxels.
    pub fn bounds(&self) -> TyVector3U32 {
        self.bounds
    }

    /// Retains a layer referencing `palette_id` after any existing ones and
    /// returns its id, back-filling every voxel with `default_material_id`.
    /// Live voxels keep `default_material_id` until
    /// [`retain_voxel`](Self::retain_voxel) overwrites it, so widening the
    /// layer set never requires re-retaining voxels. The same palette may back
    /// several layers. `default_material_id` should be one of `palette_id`'s
    /// materials; a live voxel keeping it is checked by
    /// [`VoxMain::retain_object`](crate::VoxMain::retain_object) on insert.
    pub fn retain_layer(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        default_material_id: U32Id<BVoxMaterial>,
    ) -> U32Id<BVoxLayer> {
        let layer_id = self.layer_ids.retain();
        self.layer_palette_ids.retain(layer_id, palette_id);

        self.samples.retain(
            layer_id,
            IdVec::from_vec(vec![default_material_id; self.liveness.len()]),
        );

        layer_id
    }

    /// Releases layer `id`, dropping its per-voxel sample column so every voxel
    /// keeps one fewer sample. The remaining layers keep their order. Errors,
    /// changing nothing, if `id` is not one of this object's layers. Leaves a
    /// hole until [`VoxMain::gc`](crate::VoxMain::gc) renumbers.
    pub fn release_layer(&mut self, id: U32Id<BVoxLayer>) -> Result<()> {
        if !self.layer_ids.is_retained(id) {
            return Err(Error::UnknownLayer { layer_id: id });
        }

        // Safety: a retained layer id has a value in both columns.
        unsafe { self.layer_palette_ids.release(id) };
        unsafe { self.samples.release(id) };
        self.layer_ids.release_stable(id);
        Ok(())
    }

    /// Layers in layer order, as `(layer id, palette)`. Pair a layer id with
    /// [`voxel_material`](Self::voxel_material) to read its samples.
    pub fn iter_layers(&self) -> impl Iterator<Item = (U32Id<BVoxLayer>, U32Id<BVoxPalette>)> + '_ {
        // Safety: retained layer ids have a `layer_palette_ids` value.
        self.layer_ids
            .iter()
            .map(move |layer_id| (layer_id, *unsafe { self.layer_palette_ids.get(layer_id) }))
    }

    /// Number of layers.
    pub fn layer_count(&self) -> usize {
        self.layer_ids.len()
    }

    /// The position of layer `id` in the layer order, or `None` if `id` is not
    /// one of this object's layers.
    pub fn layer_index(&self, id: U32Id<BVoxLayer>) -> Option<usize> {
        self.layer_ids.index_of(id)
    }

    /// The palette id layer `id` references, or `None` if `id` is not one of
    /// this object's layers.
    pub fn layer_palette_id(&self, id: U32Id<BVoxLayer>) -> Option<U32Id<BVoxPalette>> {
        // Safety: retained layer ids have a `layer_palette_ids` value.
        self.layer_ids
            .is_retained(id)
            .then(|| *unsafe { self.layer_palette_ids.get(id) })
    }

    /// Moves layer `id` to position `index` in the layer order, shifting the
    /// layers between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this object's layers or `index` is at or
    /// past [`layer_count`](Self::layer_count).
    pub fn move_layer(&mut self, id: U32Id<BVoxLayer>, index: usize) -> Result<()> {
        if !self.layer_ids.is_retained(id) {
            return Err(Error::UnknownLayer { layer_id: id });
        }
        let count = self.layer_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.layer_ids.move_to(id, index);
        Ok(())
    }

    /// Repoints every live voxel that samples a keyed material of
    /// `replacement_ids` through a layer referencing `palette_id` to the
    /// material it pairs with. Used by
    /// [`VoxMain::repaint_materials`](crate::VoxMain::repaint_materials).
    pub(crate) fn repaint_materials(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
    ) {
        let layer_ids: Vec<_> = self.layer_ids.iter().collect();
        for layer_id in layer_ids {
            // Safety: retained layer ids have a `layer_palette_ids` value.
            if *unsafe { self.layer_palette_ids.get(layer_id) } != palette_id {
                continue;
            }

            // Safety: retained layer ids have a sample column.
            let column = unsafe { self.samples.get_mut(layer_id) };
            for voxel_id in self.liveness.iter_live() {
                let sample = &mut column[voxel_id.to_usize_id()];
                if let Some(&replacement_id) = replacement_ids.get(sample) {
                    *sample = replacement_id;
                }
            }
        }
    }

    /// Display name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Translation from the placing hierarchy node to the grid's min corner, in
    /// voxels. `[0, 0, 0]` places the grid's min corner at the node origin.
    pub fn origin(&self) -> TyVector3I32 {
        self.origin
    }

    /// Sets the grid [`origin`](Self::origin).
    pub fn set_origin(&mut self, origin: TyVector3I32) {
        self.origin = origin;
    }

    /// Makes the voxel at `id` live with one `sample_ids` material per layer,
    /// in [`iter_layers`](Self::iter_layers) order. Errors, changing nothing,
    /// if `id` is outside the grid or `sample_ids` has the wrong length.
    pub fn retain_voxel(
        &mut self,
        id: U32Id<BVoxVoxel>,
        sample_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        if (id.to_u32() as usize) >= self.liveness.len() {
            return Err(Error::UnknownVoxel { voxel_id: id });
        }
        if sample_ids.len() != self.layer_ids.len() {
            return Err(Error::SampleArity {
                samples: sample_ids.len(),
                layers: self.layer_ids.len(),
            });
        }

        self.liveness.set_live(id, true);
        for (layer_id, &material_id) in self.layer_ids.iter().zip(sample_ids) {
            // Safety: retained layer ids have a sample column.
            let column = unsafe { self.samples.get_mut(layer_id) };
            column[id.to_usize_id()] = material_id;
        }
        Ok(())
    }

    /// Makes the voxel at `id` empty, leaving its samples in place but ignored.
    /// Errors, changing nothing, if `id` is outside the grid.
    pub fn release_voxel(&mut self, id: U32Id<BVoxVoxel>) -> Result<()> {
        if (id.to_u32() as usize) >= self.liveness.len() {
            return Err(Error::UnknownVoxel { voxel_id: id });
        }
        self.liveness.set_live(id, false);
        Ok(())
    }

    /// Whether the voxel at `id` is live. `false` if outside the grid.
    pub fn is_live(&self, id: U32Id<BVoxVoxel>) -> bool {
        (id.to_u32() as usize) < self.liveness.len() && self.liveness.is_live(id)
    }

    /// Live voxel ids in ascending raster order. Recover positions with
    /// [`voxel_position`](Self::voxel_position) and materials with
    /// [`voxel_material`](Self::voxel_material).
    pub fn iter_live(&self) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
        self.liveness.iter_live()
    }

    /// Live voxels' samples in `layer_id`, as `(voxel id, material id)`, in
    /// ascending raster order, or `None` if `layer_id` is not one of this
    /// object's layers. Reads the layer's sample column once, so a full scan
    /// skips [`voxel_material`](Self::voxel_material)'s per-call lookups.
    pub fn iter_live_samples(
        &self,
        layer_id: U32Id<BVoxLayer>,
    ) -> Option<impl Iterator<Item = (U32Id<BVoxVoxel>, U32Id<BVoxMaterial>)> + '_> {
        if !self.layer_ids.is_retained(layer_id) {
            return None;
        }
        // Safety: retained layer ids have a sample column.
        let column = unsafe { self.samples.get(layer_id) };
        Some(
            self.liveness
                .iter_live()
                .map(move |voxel_id| (voxel_id, column[voxel_id.to_usize_id()])),
        )
    }

    /// Number of live (filled) voxels.
    pub fn live_count(&self) -> usize {
        self.liveness.count_live()
    }

    /// The tight live-voxel extent as `(min_corner, [X, Y, Z] size)` in this
    /// object's grid, or `None` when it has no live voxels. The object stores
    /// the wider build volume in [`bounds`](Self::bounds).
    pub fn live_extent(&self) -> Option<(TyVector3U32, TyVector3U32)> {
        let mut live = self.iter_live().map(|voxel_id| {
            self.voxel_position(voxel_id)
                .expect("a live voxel is within the grid")
        });

        let first = live.next()?;
        let (mut min, mut max) = (first, first);
        for p in live {
            min = min.min(p);
            max = max.max(p);
        }

        Some((
            min,
            TyVector3U32::new(max.x - min.x + 1, max.y - min.y + 1, max.z - min.z + 1),
        ))
    }

    /// Voxel id at grid `position`, or `None` if outside
    /// [`bounds`](Self::bounds).
    pub fn voxel_id(&self, position: TyVector3U32) -> Option<U32Id<BVoxVoxel>> {
        if position.x >= self.bounds.x || position.y >= self.bounds.y || position.z >= self.bounds.z
        {
            return None;
        }

        // The volume cap in `new` keeps the arithmetic below within u32.
        let plane = self.bounds.y * self.bounds.z;
        Some(U32Id::from_u32(
            position.x * plane + position.y * self.bounds.z + position.z,
        ))
    }

    /// Material the live voxel `id` samples in `layer_id`, or `None` if the
    /// voxel is not live or `layer_id` is not one of this object's layers.
    pub fn voxel_material(
        &self,
        id: U32Id<BVoxVoxel>,
        layer_id: U32Id<BVoxLayer>,
    ) -> Option<U32Id<BVoxMaterial>> {
        if !self.is_live(id) || !self.layer_ids.is_retained(layer_id) {
            return None;
        }

        // Safety: retained layer ids have a sample column.
        let column = unsafe { self.samples.get(layer_id) };
        Some(column[id.to_usize_id()])
    }

    /// Grid position of `id`, or `None` if outside the grid. Inverse of
    /// [`voxel_id`](Self::voxel_id).
    pub fn voxel_position(&self, id: U32Id<BVoxVoxel>) -> Option<TyVector3U32> {
        let raster = id.to_u32();
        if (raster as u64) >= Self::volume_of(self.bounds) {
            return None;
        }

        // The volume cap in `new` keeps `plane` within u32. A non-zero volume
        // guarantees both divisors below are non-zero.
        let plane = self.bounds.y * self.bounds.z;
        Some(TyVector3U32::new(
            raster / plane,
            (raster % plane) / self.bounds.z,
            raster % self.bounds.z,
        ))
    }
}

impl Drop for VoxObject {
    fn drop(&mut self) {
        // Safety: every `layer_ids` id has a value in both columns.
        unsafe {
            self.layer_palette_ids.release_all(&self.layer_ids);
            self.samples.release_all(&self.layer_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxMaterial, BVoxPalette, Error, VoxObject};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    fn material_id(index: u32) -> U32Id<BVoxMaterial> {
        U32Id::from_u32(index)
    }

    #[test]
    fn voxel_id_and_position_round_trip_and_bound_check() {
        let object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 3, 4)).unwrap();
        let position = TyVector3U32::new(1, 2, 3);
        let voxel_id = object.voxel_id(position).unwrap();
        assert_eq!(voxel_id.to_u32(), 23); // 1*(3*4) + 2*4 + 3
        assert_eq!(object.voxel_position(voxel_id), Some(position));
        // Out of bounds yields None rather than erroring.
        assert_eq!(object.voxel_id(TyVector3U32::new(2, 0, 0)), None);
        assert_eq!(object.voxel_position(U32Id::from_u32(24)), None);
    }

    #[test]
    fn new_rejects_grid_past_the_cell_cap() {
        // 2048^3 = 2^33 cells, well past MAX_GRID_CELLS (2^27).
        assert_eq!(
            VoxObject::new("huge".to_owned(), TyVector3U32::new(2048, 2048, 2048)).unwrap_err(),
            Error::GridCellCap { cells: 1 << 33 }
        );
    }

    #[test]
    fn new_accepts_a_grid_at_the_cell_cap() {
        assert!(VoxObject::new("max".to_owned(), TyVector3U32::new(512, 512, 512)).is_ok());
    }

    #[test]
    fn retain_and_release_track_liveness_and_samples() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let voxel_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();

        assert!(!object.is_live(voxel_id));
        assert_eq!(object.voxel_material(voxel_id, layer_id), None);

        assert_eq!(object.retain_voxel(voxel_id, &[material_id(7)]), Ok(()));
        assert!(object.is_live(voxel_id));
        assert_eq!(object.live_count(), 1);
        assert_eq!(
            object.voxel_material(voxel_id, layer_id),
            Some(material_id(7))
        );
        assert_eq!(object.iter_live().collect::<Vec<_>>(), [voxel_id]);

        assert_eq!(object.release_voxel(voxel_id), Ok(()));
        assert!(!object.is_live(voxel_id));
        assert_eq!(object.live_count(), 0);
        assert_eq!(object.voxel_material(voxel_id, layer_id), None);
    }

    #[test]
    fn retain_voxel_rejects_bad_input_without_changing_state() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

        // Wrong sample arity and an out-of-grid id are both rejected,
        // untouched.
        assert_eq!(
            object.retain_voxel(voxel_id, &[]),
            Err(Error::SampleArity {
                samples: 0,
                layers: 1
            })
        );
        assert_eq!(
            object.retain_voxel(U32Id::from_u32(99), &[material_id(0)]),
            Err(Error::UnknownVoxel {
                voxel_id: U32Id::from_u32(99)
            })
        );
        assert_eq!(object.live_count(), 0);
        assert_eq!(
            object.release_voxel(U32Id::from_u32(99)),
            Err(Error::UnknownVoxel {
                voxel_id: U32Id::from_u32(99)
            })
        );
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
            let voxel_id = object.voxel_id(position).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }

        let live: Vec<(u32, [u32; 3])> = object
            .iter_live()
            .map(|voxel_id| {
                let position = object.voxel_position(voxel_id).unwrap();
                (voxel_id.to_u32(), [position.x, position.y, position.z])
            })
            .collect();
        assert_eq!(live, [(0, [0, 0, 0]), (6, [0, 1, 2]), (23, [1, 2, 3])]);
    }

    #[test]
    fn iter_live_samples_walks_a_layer_in_raster_order() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let first_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let second_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(second_id, &[material_id(7)]).unwrap();
        object.retain_voxel(first_id, &[material_id(2)]).unwrap();

        let samples: Vec<_> = object.iter_live_samples(layer_id).unwrap().collect();
        assert_eq!(
            samples,
            [(first_id, material_id(2)), (second_id, material_id(7))]
        );
        // A layer id the object never minted is rejected.
        assert!(object.iter_live_samples(U32Id::from_u32(9)).is_none());
    }

    #[test]
    fn clone_object_is_an_independent_deep_copy() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(3), material_id(0));
        let voxel_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[material_id(7)]).unwrap();

        let copy = object.clone_object();
        assert_eq!(copy.name(), "o");
        assert_eq!(copy.bounds(), TyVector3U32::new(2, 1, 1));
        assert_eq!(copy.live_count(), 1);
        assert_eq!(
            copy.voxel_material(voxel_id, layer_id),
            Some(material_id(7))
        );
        assert_eq!(
            copy.iter_layers().collect::<Vec<_>>(),
            [(layer_id, U32Id::<BVoxPalette>::from_u32(3))]
        );

        // Editing the original must not touch the copy.
        object.release_voxel(voxel_id).unwrap();
        assert_eq!(object.live_count(), 0);
        assert_eq!(copy.live_count(), 1);
    }

    #[test]
    fn two_layers_may_share_a_palette() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // Two layers referencing the same palette is allowed; layers do not
        // merge.
        let first_id = object.retain_layer(palette_id, material_id(0));
        let second_id = object.retain_layer(palette_id, material_id(0));
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[material_id(2), material_id(5)])
            .unwrap();

        assert_eq!(object.layer_count(), 2);
        assert_eq!(
            object.voxel_material(voxel_id, first_id),
            Some(material_id(2))
        );
        assert_eq!(
            object.voxel_material(voxel_id, second_id),
            Some(material_id(5))
        );
        assert_eq!(
            object.iter_layers().collect::<Vec<_>>(),
            [(first_id, palette_id), (second_id, palette_id)]
        );
        assert_eq!(object.layer_palette_id(first_id), Some(palette_id));
        assert_eq!(object.layer_palette_id(U32Id::from_u32(9)), None);
    }

    #[test]
    fn release_layer_preserves_the_survivors_order() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let first_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let middle_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(1), material_id(0));
        let last_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(2), material_id(0));

        // Releasing the first of three is the smallest case a swap-remove would
        // get wrong, listing `last_id` before `middle_id`.
        assert_eq!(object.release_layer(first_id), Ok(()));
        assert_eq!(
            object.iter_layers().collect::<Vec<_>>(),
            [
                (middle_id, U32Id::<BVoxPalette>::from_u32(1)),
                (last_id, U32Id::<BVoxPalette>::from_u32(2)),
            ]
        );

        // A layer retained after the release appends at the end of the order.
        let added_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(3), material_id(0));
        assert_eq!(
            object
                .iter_layers()
                .map(|(layer_id, _)| layer_id)
                .collect::<Vec<_>>(),
            [middle_id, last_id, added_id]
        );
    }

    #[test]
    fn move_layer_reorders_the_listing_and_validates() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let first_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let second_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(1), material_id(0));
        let third_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(2), material_id(0));
        assert_eq!(object.layer_index(second_id), Some(1));

        assert_eq!(object.move_layer(third_id, 0), Ok(()));
        assert_eq!(
            object
                .iter_layers()
                .map(|(layer_id, _)| layer_id)
                .collect::<Vec<_>>(),
            [third_id, first_id, second_id]
        );
        assert_eq!(object.layer_index(third_id), Some(0));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            object.move_layer(third_id, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        assert_eq!(
            object.move_layer(U32Id::from_u32(9), 0),
            Err(Error::UnknownLayer {
                layer_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(object.layer_index(U32Id::from_u32(9)), None);
        assert_eq!(
            object
                .iter_layers()
                .map(|(layer_id, _)| layer_id)
                .collect::<Vec<_>>(),
            [third_id, first_id, second_id]
        );
    }

    #[test]
    fn release_layer_drops_its_samples_leaving_others() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let first_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let second_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(1), material_id(0));
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[material_id(5), material_id(6)])
            .unwrap();

        assert_eq!(object.release_layer(first_id), Ok(()));
        assert_eq!(object.layer_count(), 1);
        assert_eq!(
            object.release_layer(first_id),
            Err(Error::UnknownLayer { layer_id: first_id })
        ); // already gone

        // The surviving layer still resolves to palette 1, material 6, and a
        // voxel now expects exactly one sample.
        assert_eq!(
            object.voxel_material(voxel_id, second_id),
            Some(material_id(6))
        );
        assert_eq!(
            object.iter_layers().collect::<Vec<_>>(),
            [(second_id, U32Id::<BVoxPalette>::from_u32(1))]
        );
        assert_eq!(object.retain_voxel(voxel_id, &[material_id(6)]), Ok(()));
    }
}
