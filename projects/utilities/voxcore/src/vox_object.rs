use crate::{BVoxLayer, BVoxMaterial, BVoxPalette, BVoxVoxel, VoxLiveness};
use branded_id::{
    IdVec, U32Id,
    soa::{IdField, IdRemap, IdStruct},
};
use ty_math::{TyVector3I32, TyVector3U32};

/// One object's voxel volume: a dense grid, the ordered layers it references,
/// and the material each voxel samples in each layer.
///
/// Each layer references a shared [`VoxPalette`](crate::VoxPalette), and
/// layers override back to front: each property takes its value from the
/// last layer that supplies it. An unsampled layer's sample cells are
/// ignored filler (see
/// [`VoxMain::layer_is_sampled`](crate::VoxMain::layer_is_sampled)).
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

    /// One id per grid cell; id equals the raster index.
    voxel_ids: IdStruct<BVoxVoxel>,

    /// Which cells are filled, one bit per voxel id.
    liveness: VoxLiveness,

    /// Pool shared by `layer_palettes` and `samples`.
    layer_ids: IdStruct<BVoxLayer>,

    /// The palette each layer references, in layer order.
    layer_palettes: IdField<BVoxLayer, U32Id<BVoxPalette>>,

    /// Per layer, the material each voxel samples. Cells of non-live voxels
    /// and of unsampled layers are ignored filler.
    samples: IdField<BVoxLayer, IdField<BVoxVoxel, U32Id<BVoxMaterial>>>,
}

impl VoxObject {
    /// Largest dense grid an object may allocate, in cells. The grid stores
    /// every cell whether live or not, so this caps memory. Always
    /// `<= u32::MAX`, so a voxel id is always a valid raster index.
    pub const MAX_GRID_CELLS: u64 = 1 << 27;

    /// Creates an empty grid of size `bounds`: every cell has a voxel id, none
    /// is live, and no layers are referenced yet. Then use
    /// [`add_layer`](Self::add_layer) and [`retain_voxel`](Self::retain_voxel).
    /// `None` if the grid would exceed
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
            origin: TyVector3I32::default(),
            voxel_ids,
            liveness: VoxLiveness::new(volume as usize),
            layer_ids: IdStruct::new(),
            layer_palettes: IdField::new(),
            samples: IdField::new(),
        })
    }

    /// Cell count `X*Y*Z`, saturating.
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

    /// Translation from the placing hierarchy node to the grid's min corner, in
    /// voxels. `[0, 0, 0]` places the grid's min corner at the node origin.
    pub fn origin(&self) -> TyVector3I32 {
        self.origin
    }

    /// Sets the grid [`origin`](Self::origin).
    pub fn set_origin(&mut self, origin: TyVector3I32) {
        self.origin = origin;
    }

    /// Number of live (filled) voxels.
    pub fn live_count(&self) -> usize {
        self.liveness.count_live()
    }

    /// Voxel id at grid `position`, or `None` if outside
    /// [`bounds`](Self::bounds).
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

    /// Material the live voxel `id` samples in `layer`, or `None` if the voxel
    /// is not live or `layer` is not one of this object's layers. For an
    /// unsampled layer this reads an ignored filler cell.
    pub fn voxel_material(
        &self,
        id: U32Id<BVoxVoxel>,
        layer: U32Id<BVoxLayer>,
    ) -> Option<U32Id<BVoxMaterial>> {
        if !self.is_live(id) || !self.layer_ids.is_retained(layer) {
            return None;
        }

        // Safety: the dense grid gives every layer a slot for every voxel id.
        let column = unsafe { self.samples.get(layer) };
        Some(*unsafe { column.get(id) })
    }

    /// Live voxel ids in ascending raster order. Recover positions with
    /// [`voxel_position`](Self::voxel_position) and materials with
    /// [`voxel_material`](Self::voxel_material).
    pub fn iter_live(&self) -> impl Iterator<Item = U32Id<BVoxVoxel>> + '_ {
        self.liveness.iter_live()
    }

    /// Live voxels' samples in `layer`, as `(voxel id, material)`, in ascending
    /// raster order, or `None` if `layer` is not one of this object's layers.
    /// Reads the layer's sample column once, so a full scan skips
    /// [`voxel_material`](Self::voxel_material)'s per-call lookups. For an
    /// unsampled layer this walks ignored filler cells.
    pub fn iter_live_samples(
        &self,
        layer: U32Id<BVoxLayer>,
    ) -> Option<impl Iterator<Item = (U32Id<BVoxVoxel>, U32Id<BVoxMaterial>)> + '_> {
        if !self.layer_ids.is_retained(layer) {
            return None;
        }
        // Safety: retained layer ids have a sample column.
        let column = unsafe { self.samples.get(layer) };
        Some(self.liveness.iter_live().map(move |id| {
            // Safety: the dense grid gives every layer a slot for every voxel
            // id.
            (id, *unsafe { column.get(id) })
        }))
    }

    /// The tight live-voxel extent as `(min_corner, [X, Y, Z] size)` in this
    /// object's grid, or `None` when it has no live voxels. The object stores
    /// the wider build volume in [`bounds`](Self::bounds); the runtime/tight
    /// grid a Voxel Json document records is derived from this.
    pub fn live_extent(&self) -> Option<(TyVector3U32, TyVector3U32)> {
        let mut live = self.iter_live().map(|id| {
            self.voxel_position(id)
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

    /// Number of layers.
    pub fn layer_count(&self) -> usize {
        self.layer_ids.len()
    }

    /// The palette `layer` references, or `None` if `layer` is not one of this
    /// object's layers.
    pub fn layer_palette(&self, id: U32Id<BVoxLayer>) -> Option<U32Id<BVoxPalette>> {
        // Safety: retained layer ids have a `layer_palettes` value.
        self.layer_ids
            .is_retained(id)
            .then(|| *unsafe { self.layer_palettes.get(id) })
    }

    /// Layers in layer order, as `(layer id, palette)`. Pair a layer id with
    /// [`voxel_material`](Self::voxel_material) to read its samples.
    pub fn iter_layers(&self) -> impl Iterator<Item = (U32Id<BVoxLayer>, U32Id<BVoxPalette>)> + '_ {
        // Safety: retained layer ids have a `layer_palettes` value.
        self.layer_ids
            .iter()
            .map(move |id| (id, *unsafe { self.layer_palettes.get(id) }))
    }

    /// Adds a layer referencing `palette` after any existing ones and returns
    /// its id, back-filling every voxel with `default_material`. Live voxels
    /// keep `default_material` until [`retain_voxel`](Self::retain_voxel)
    /// overwrites it, so widening the layer set never requires re-adding voxels.
    /// The same palette may back several layers. If `palette` has no
    /// materials the layer is unsampled and every cell is ignored filler, so
    /// `default_material` may be any id.
    pub fn add_layer(
        &mut self,
        palette: U32Id<BVoxPalette>,
        default_material: U32Id<BVoxMaterial>,
    ) -> U32Id<BVoxLayer> {
        let layer_id = self.layer_ids.retain();
        self.layer_palettes.retain(layer_id, palette);

        let mut column = IdField::new();
        for voxel_id in &self.voxel_ids {
            column.retain(voxel_id, default_material);
        }

        self.samples.retain(layer_id, column);

        layer_id
    }

    /// Makes the voxel at `id` live with one `samples` material per layer, in
    /// [`add_layer`](Self::add_layer) order, unsampled layers included; their
    /// entries are stored as ignored filler. `None`, changing nothing, if
    /// `id` is outside the grid or `samples` has the wrong length.
    pub fn retain_voxel(
        &mut self,
        id: U32Id<BVoxVoxel>,
        samples: &[U32Id<BVoxMaterial>],
    ) -> Option<()> {
        if (id.to_u32() as usize) >= self.liveness.len() || samples.len() != self.layer_ids.len() {
            return None;
        }

        self.liveness.set_live(id, true);
        for (layer_id, &material) in self.layer_ids.iter().zip(samples) {
            // Safety: the dense grid gives every layer a slot for `id`.
            let column = unsafe { self.samples.get_mut(layer_id) };
            unsafe { column.set(id, material) };
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
        // `layer_palettes` is Copy-valued and clones wholesale below.
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
            voxel_ids: self.voxel_ids.clone(),
            liveness: self.liveness.clone(),
            layer_ids: self.layer_ids.clone(),
            layer_palettes: self.layer_palettes.clone(),
            samples,
        }
    }

    /// Removes layer `id`, dropping its per-voxel sample column so every voxel
    /// keeps one fewer sample. `None`, changing nothing, if `id` is not one of
    /// this object's layers. Leaves a hole until
    /// [`VoxMain::gc`](crate::VoxMain::gc) renumbers.
    pub fn remove_layer(&mut self, id: U32Id<BVoxLayer>) -> Option<()> {
        if !self.layer_ids.is_retained(id) {
            return None;
        }

        // Safety: a retained layer id has a value in both columns. Sample
        // materials are Copy, so dropping the inner column frees its storage
        // with nothing to release per voxel.
        unsafe { self.layer_palettes.release(id) };
        unsafe { self.samples.release(id) };
        self.layer_ids.release(id);
        Some(())
    }

    /// Removes every layer referencing `palette`. Two layers may reference the
    /// same palette, so this removes every match. Used by
    /// [`VoxMain::remove_palette`](crate::VoxMain::remove_palette) to detach an
    /// object from a palette being removed.
    pub(crate) fn remove_layers_to(&mut self, palette: U32Id<BVoxPalette>) {
        let doomed: Vec<_> = self
            .iter_layers()
            .filter(|&(_, p)| p == palette)
            .map(|(id, _)| id)
            .collect();

        for id in doomed {
            self.remove_layer(id);
        }
    }

    /// Repoints every live voxel that samples `old` through a layer referencing
    /// `palette` to `new`. Used by
    /// [`VoxMain::remove_material`](crate::VoxMain::remove_material) before the
    /// old material is dropped.
    pub(crate) fn repaint_material(
        &mut self,
        palette: U32Id<BVoxPalette>,
        old: U32Id<BVoxMaterial>,
        new: U32Id<BVoxMaterial>,
    ) {
        let layer_ids: Vec<_> = self.layer_ids.iter().collect();
        let live: Vec<_> = self.liveness.iter_live().collect();
        for layer_id in layer_ids {
            // Safety: retained layer ids have a `layer_palettes` value.
            if *unsafe { self.layer_palettes.get(layer_id) } != palette {
                continue;
            }

            // Safety: retained layer ids have a sample column.
            let column = unsafe { self.samples.get_mut(layer_id) };
            for &voxel_id in &live {
                // Safety: the dense grid gives every layer a slot for every
                // voxel id.
                if *unsafe { column.get(voxel_id) } == old {
                    *unsafe { column.get_mut(voxel_id) } = new;
                }
            }
        }
    }

    /// Rewrites this object's cross-references to match pools a
    /// [`VoxMain`](crate::VoxMain) is compacting, then compacts its own layer
    /// pool. Each layer's palette is translated through `palette_remap`, and
    /// each sampled layer's live-voxel sample materials through the
    /// `material_remaps` entry for the referenced palette's pre-gc id; an
    /// unsampled layer's cells are filler and left untouched. Requires a
    /// referentially valid object, so every translation resolves.
    pub(crate) fn gc(
        &mut self,
        palette_remap: &IdRemap<BVoxPalette, u32>,
        material_remaps: &IdVec<BVoxPalette, IdRemap<BVoxMaterial, u32>>,
    ) {
        let layer_ids: Vec<_> = self.layer_ids.iter().collect();
        let live: Vec<_> = self.liveness.iter_live().collect();
        for layer_id in layer_ids {
            // Translate the referenced palette id to its relabeled value.
            // Safety: retained layer ids have a `layer_palettes` value.
            let old_palette = *unsafe { self.layer_palettes.get(layer_id) };
            let new_palette = palette_remap
                .new_id(old_palette)
                .expect("a layer references a live palette in a valid state");
            // Safety: same retained layer id.
            *unsafe { self.layer_palettes.get_mut(layer_id) } = new_palette;

            // Translate each live voxel's sample material through that palette's
            // relabeling. Filler cells are exempt: non-live voxels', and every
            // cell of an unsampled layer, whose palette has no materials and so
            // an empty relabeling.
            let material_remap = &material_remaps[old_palette.to_usize_id()];
            if material_remap.is_empty() {
                continue;
            }
            // Safety: retained layer ids have a sample column.
            let column = unsafe { self.samples.get_mut(layer_id) };
            for &voxel_id in &live {
                // Safety: the dense grid gives every layer a slot for every
                // voxel id.
                let material = *unsafe { column.get(voxel_id) };
                let new_material = material_remap
                    .new_id(material)
                    .expect("a live voxel samples a live material in a valid state");
                // Safety: same dense slot.
                *unsafe { column.get_mut(voxel_id) } = new_material;
            }
        }

        // Compact the layer pool; the values above were already translated, so
        // this only relabels layer keys.
        let layer_remap = self.layer_ids.gc();
        // Safety: both columns were in sync with the pre-gc layer pool, and
        // nothing has retained or released since.
        unsafe { self.samples.gc(&layer_remap) };
        unsafe { self.layer_palettes.gc(&layer_remap) };
    }
}

impl Drop for VoxObject {
    fn drop(&mut self) {
        // Safety: every `layer_ids` id has a value in both columns. Sample
        // materials are Copy, so dropping the inner columns frees their storage
        // and the voxel pool needs no part here.
        unsafe {
            self.layer_palettes.release_all(&self.layer_ids);
            self.samples.release_all(&self.layer_ids);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxMaterial, BVoxPalette, VoxObject};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    fn material(index: u32) -> U32Id<BVoxMaterial> {
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

    // The at-cap case allocates 512^3 voxel ids, which Miri interprets far too
    // slowly; the cap arithmetic exercises no unsafe, so skip it under Miri.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn new_rejects_grid_past_the_cell_cap() {
        // 2048^3 = 2^33 cells, well past MAX_GRID_CELLS (2^27).
        assert!(VoxObject::new("huge".to_owned(), TyVector3U32::new(2048, 2048, 2048)).is_none());
        assert!(VoxObject::new("max".to_owned(), TyVector3U32::new(512, 512, 512)).is_some());
    }

    #[test]
    fn retain_and_release_track_liveness_and_samples() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer = object.add_layer(U32Id::<BVoxPalette>::from_u32(0), material(0));
        let id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();

        assert!(!object.is_live(id));
        assert_eq!(object.voxel_material(id, layer), None);

        assert_eq!(object.retain_voxel(id, &[material(7)]), Some(()));
        assert!(object.is_live(id));
        assert_eq!(object.live_count(), 1);
        assert_eq!(object.voxel_material(id, layer), Some(material(7)));
        assert_eq!(object.iter_live().collect::<Vec<_>>(), [id]);

        assert_eq!(object.release_voxel(id), Some(()));
        assert!(!object.is_live(id));
        assert_eq!(object.live_count(), 0);
        assert_eq!(object.voxel_material(id, layer), None);
    }

    #[test]
    fn retain_voxel_rejects_bad_input_without_changing_state() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(U32Id::<BVoxPalette>::from_u32(0), material(0));
        let id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();

        // Wrong sample arity and an out-of-grid id are both rejected,
        // untouched.
        assert_eq!(object.retain_voxel(id, &[]), None);
        assert_eq!(
            object.retain_voxel(U32Id::from_u32(99), &[material(0)]),
            None
        );
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
    fn iter_live_samples_walks_a_layer_in_raster_order() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer = object.add_layer(U32Id::<BVoxPalette>::from_u32(0), material(0));
        let first = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let second = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(second, &[material(7)]).unwrap();
        object.retain_voxel(first, &[material(2)]).unwrap();

        let samples: Vec<_> = object.iter_live_samples(layer).unwrap().collect();
        assert_eq!(samples, [(first, material(2)), (second, material(7))]);
        // A layer id the object never minted is rejected.
        assert!(object.iter_live_samples(U32Id::from_u32(9)).is_none());
    }

    #[test]
    fn clone_object_is_an_independent_deep_copy() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        let layer = object.add_layer(U32Id::<BVoxPalette>::from_u32(3), material(0));
        let id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object.retain_voxel(id, &[material(7)]).unwrap();

        let copy = object.clone_object();
        assert_eq!(copy.name(), "o");
        assert_eq!(copy.bounds(), TyVector3U32::new(2, 1, 1));
        assert_eq!(copy.live_count(), 1);
        assert_eq!(copy.voxel_material(id, layer), Some(material(7)));
        assert_eq!(
            copy.iter_layers().collect::<Vec<_>>(),
            [(layer, U32Id::<BVoxPalette>::from_u32(3))]
        );

        // Editing the original must not touch the copy.
        object.release_voxel(id).unwrap();
        assert_eq!(object.live_count(), 0);
        assert_eq!(copy.live_count(), 1);
    }

    #[test]
    fn two_layers_may_share_a_palette() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let palette = U32Id::<BVoxPalette>::from_u32(0);
        // Two layers referencing the same palette is allowed; layers do not
        // merge.
        let first = object.add_layer(palette, material(0));
        let second = object.add_layer(palette, material(0));
        let id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(id, &[material(2), material(5)])
            .unwrap();

        assert_eq!(object.layer_count(), 2);
        assert_eq!(object.voxel_material(id, first), Some(material(2)));
        assert_eq!(object.voxel_material(id, second), Some(material(5)));
        assert_eq!(
            object.iter_layers().collect::<Vec<_>>(),
            [(first, palette), (second, palette)]
        );
        assert_eq!(object.layer_palette(first), Some(palette));
        assert_eq!(object.layer_palette(U32Id::from_u32(9)), None);
    }

    #[test]
    fn remove_layer_drops_its_samples_leaving_others() {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let first = object.add_layer(U32Id::<BVoxPalette>::from_u32(0), material(0));
        let second = object.add_layer(U32Id::<BVoxPalette>::from_u32(1), material(0));
        let id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(id, &[material(5), material(6)])
            .unwrap();

        assert_eq!(object.remove_layer(first), Some(()));
        assert_eq!(object.layer_count(), 1);
        assert_eq!(object.remove_layer(first), None); // already gone

        // The surviving layer still resolves to palette 1, material 6, and a
        // voxel now expects exactly one sample.
        assert_eq!(object.voxel_material(id, second), Some(material(6)));
        assert_eq!(
            object.iter_layers().collect::<Vec<_>>(),
            [(second, U32Id::<BVoxPalette>::from_u32(1))]
        );
        assert_eq!(object.retain_voxel(id, &[material(6)]), Some(()));
    }
}
