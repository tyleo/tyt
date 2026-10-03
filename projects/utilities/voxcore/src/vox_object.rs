use crate::{BVoxLayer, BVoxMaterial, BVoxPalette, BVoxVoxel, Error, Result, VoxLiveness};
use branded_id::{
    IdRange, IdVec, U32Id,
    soa::{IdField, IdRemap, IdStruct, IdStructView, IdStructViewMut},
};
use std::collections::HashMap;
use ty_math::{TyVector3I32, TyVector3U32};

/// The layer column of palette ids.
type LayerPaletteIdColumn = IdField<BVoxLayer, U32Id<BVoxPalette>>;

/// The layer column of samples.
type LayerSampleColumn = IdField<BVoxLayer, IdVec<BVoxVoxel, U32Id<BVoxMaterial>>>;

/// One object's voxel volume: a dense grid, the ordered layers it references,
/// and the material each voxel samples in each layer.
///
/// Each layer references a shared [`VoxPalette`](crate::VoxPalette), and layers
/// override back to front: each property takes its value from the last layer
/// that supplies it.
///
/// The grid sits on glTF's frame: Y-up, right-handed, with +Z toward the
/// viewer. A live voxel at `(x, y, z)` fills the unit cube whose min corner
/// is that position, in voxel units.
///
/// Every grid cell has a voxel id equal to its raster index `x*Y*Z + y*Z + z`,
/// so [`voxel_id`](Self::voxel_id) and [`voxel_position`](Self::voxel_position)
/// interconvert. [`is_live`](Self::is_live) says which cells are filled.
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
    layer_palette_ids: LayerPaletteIdColumn,

    /// Per layer, the material each voxel samples, one slot per grid cell.
    /// Cells of non-live voxels are ignored filler.
    samples: LayerSampleColumn,
}

impl VoxObject {
    /// Largest dense grid an object may allocate, in cells. The grid stores
    /// every cell whether live or not, so this caps memory. Because the cap is
    /// `<= u32::MAX`, a voxel id is always a valid raster index.
    pub const MAX_GRID_CELLS: u64 = 1 << 27;

    /// Creates an empty grid of size `bounds`: every cell has a voxel id, none
    /// is live, and no layers are referenced yet. Then use
    /// [`retain_layer`](Self::retain_layer) and
    /// [`retain_voxel`](Self::retain_voxel). Errors, building nothing, if the
    /// grid would exceed [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS).
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

    /// Cell count `X*Y*Z` of a grid of `bounds`, saturating.
    pub fn volume_of(bounds: TyVector3U32) -> u64 {
        (bounds.x as u64)
            .saturating_mul(bounds.y as u64)
            .saturating_mul(bounds.z as u64)
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
        let live_ids: Vec<_> = self.liveness.iter_live().collect();

        // Samples translate while the layer still holds the palette's pre-gc
        // id. The filler cells of non-live voxels are exempt.
        for (_, (palette_id, samples)) in self.layers_mut() {
            let material_remap = &material_remaps[palette_id.to_usize_id()];

            for &voxel_id in &live_ids {
                let sample = &mut samples[voxel_id.to_usize_id()];

                *sample = material_remap
                    .new_id(*sample)
                    .expect("a live voxel samples a live material in a valid state");
            }
        }

        self.relabel_layer_palettes(|palette_id| {
            palette_remap
                .new_id(palette_id)
                .expect("a layer references a live palette in a valid state")
        });

        // Compact the layer id pool; the values above were already translated,
        // so this only relabels layer keys.
        self.layers_mut().gc();
    }

    /// Translates every layer's palette id through `palette_id`, for an object
    /// moving to another [`VoxMain`](crate::VoxMain)'s palettes. Material ids
    /// are palette-local, so the samples keep them.
    /// [`VoxMain::retain_object`](crate::VoxMain::retain_object) checks the
    /// new ids on insert.
    pub fn relabel_layer_palettes(
        &mut self,
        mut palette_id: impl FnMut(U32Id<BVoxPalette>) -> U32Id<BVoxPalette>,
    ) {
        for (_, (layer_palette_id, _)) in self.layers_mut() {
            *layer_palette_id = palette_id(*layer_palette_id);
        }
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
        let samples = IdVec::from_vec(vec![default_material_id; self.liveness.len()]);

        self.layers_mut().retain((palette_id, samples))
    }

    /// Releases layer `id`, dropping its per-voxel sample column so every voxel
    /// keeps one fewer sample. The remaining layers keep their order. Errors,
    /// changing nothing, if `id` is not one of this object's layers. Leaves a
    /// hole until [`VoxMain::gc`](crate::VoxMain::gc) renumbers.
    pub fn release_layer(&mut self, id: U32Id<BVoxLayer>) -> Result<()> {
        let Some(_) = self.layers_mut().release_stable(id) else {
            return Err(Error::UnknownLayer { layer_id: id });
        };

        Ok(())
    }

    /// Layers in layer order, as `(layer id, palette id)`. Pair a layer id with
    /// [`voxel_material`](Self::voxel_material) to read its samples.
    pub fn iter_layers(&self) -> impl Iterator<Item = (U32Id<BVoxLayer>, U32Id<BVoxPalette>)> + '_ {
        self.layers()
            .into_iter()
            .map(|(layer_id, (palette_id, _))| (layer_id, *palette_id))
    }

    /// Number of layers.
    pub fn layer_count(&self) -> usize {
        self.layer_ids.len()
    }

    /// The palette id layer `id` references, or `None` if `id` is not one of
    /// this object's layers.
    pub fn layer_palette_id(&self, id: U32Id<BVoxLayer>) -> Option<U32Id<BVoxPalette>> {
        let (palette_id, _) = self.layers().get(id)?;

        Some(*palette_id)
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
        let live_ids: Vec<_> = self.liveness.iter_live().collect();

        for (_, (&mut layer_palette_id, samples)) in self.layers_mut() {
            if layer_palette_id != palette_id {
                continue;
            }

            for &voxel_id in &live_ids {
                let sample = &mut samples[voxel_id.to_usize_id()];

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

    /// Sets the display [`name`](Self::name).
    pub fn set_name(&mut self, name: String) {
        self.name = name;
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

        for ((_, (_, samples)), &material_id) in self.layers_mut().into_iter().zip(sample_ids) {
            samples[id.to_usize_id()] = material_id;
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
    /// object's layers. Reads the layer's sample column once, so a full scan is
    /// cheaper than per-voxel [`voxel_material`](Self::voxel_material) calls.
    pub fn iter_live_samples(
        &self,
        layer_id: U32Id<BVoxLayer>,
    ) -> Option<impl Iterator<Item = (U32Id<BVoxVoxel>, U32Id<BVoxMaterial>)> + '_> {
        let (_, samples) = self.layers().get(layer_id)?;

        Some(
            self.liveness
                .iter_live()
                .map(move |voxel_id| (voxel_id, samples[voxel_id.to_usize_id()])),
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
        Self::raster_id(self.bounds, position)
    }

    /// The voxel id of `position` on any grid of `bounds`, or `None` if
    /// `position` is outside the grid or the grid exceeds
    /// [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS).
    pub fn raster_id(bounds: TyVector3U32, position: TyVector3U32) -> Option<U32Id<BVoxVoxel>> {
        // The cap keeps the arithmetic below within u32.
        if Self::volume_of(bounds) > Self::MAX_GRID_CELLS
            || position.x >= bounds.x
            || position.y >= bounds.y
            || position.z >= bounds.z
        {
            return None;
        }

        let plane = bounds.y * bounds.z;
        Some(U32Id::from_u32(
            position.x * plane + position.y * bounds.z + position.z,
        ))
    }

    /// Material the live voxel `id` samples in `layer_id`, or `None` if the
    /// voxel is not live or `layer_id` is not one of this object's layers.
    pub fn voxel_material(
        &self,
        id: U32Id<BVoxVoxel>,
        layer_id: U32Id<BVoxLayer>,
    ) -> Option<U32Id<BVoxMaterial>> {
        if !self.is_live(id) {
            return None;
        }

        let (_, samples) = self.layers().get(layer_id)?;

        Some(samples[id.to_usize_id()])
    }

    /// Grid position of `id`, or `None` if outside the grid. Inverse of
    /// [`voxel_id`](Self::voxel_id).
    pub fn voxel_position(&self, id: U32Id<BVoxVoxel>) -> Option<TyVector3U32> {
        Self::raster_position(self.bounds, id)
    }

    /// The position of the voxel id `id` on any grid of `bounds`, or `None`
    /// if `id` is outside the grid or the grid exceeds
    /// [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS). Inverse of
    /// [`raster_id`](Self::raster_id).
    pub fn raster_position(bounds: TyVector3U32, id: U32Id<BVoxVoxel>) -> Option<TyVector3U32> {
        let raster = id.to_u32();
        let volume = Self::volume_of(bounds);

        // The cap keeps the arithmetic below within u32.
        if volume > Self::MAX_GRID_CELLS || u64::from(raster) >= volume {
            return None;
        }

        // A non-zero volume guarantees both divisors below are non-zero.
        let plane = bounds.y * bounds.z;
        Some(TyVector3U32::new(
            raster / plane,
            (raster % plane) / bounds.z,
            raster % bounds.z,
        ))
    }

    /// Moves every live voxel onto a grid of `bounds`, the voxel at `p` landing
    /// at `position(p)` with its samples. Layers and `origin` stay. Returns the
    /// new id of each live voxel keyed by its old id. Errors, changing nothing,
    /// if:
    ///
    /// 1. the grid would exceed [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS)
    /// 2. a live voxel lands outside the grid
    /// 3. two live voxels land on one cell
    pub fn remap_voxels(
        &mut self,
        bounds: TyVector3U32,
        position: impl Fn(TyVector3U32) -> TyVector3I32,
    ) -> Result<HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>> {
        let volume = Self::volume_of(bounds);
        if volume > Self::MAX_GRID_CELLS {
            return Err(Error::GridCellCap { cells: volume });
        }

        let mut liveness = VoxLiveness::new(volume as usize);
        let mut new_ids = HashMap::with_capacity(self.live_count());
        let mut old_ids = HashMap::with_capacity(self.live_count());
        for voxel_id in self.liveness.iter_live() {
            let old_position = self
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let new_position = position(old_position);
            let Some(new_id) = TyVector3U32::try_from(new_position)
                .ok()
                .and_then(|new_position| Self::raster_id(bounds, new_position))
            else {
                return Err(Error::RemappedVoxelOutsideGrid {
                    voxel_id,
                    position: new_position,
                    bounds,
                });
            };

            if let Some(&other_voxel_id) = old_ids.get(&new_id) {
                return Err(Error::RemappedVoxelCollision {
                    voxel_id,
                    other_voxel_id,
                });
            }

            liveness.set_live(new_id, true);
            old_ids.insert(new_id, voxel_id);
            new_ids.insert(voxel_id, new_id);
        }

        // Non-live cells are ignored filler, so material 0 stands in.
        for (_, (_, samples)) in self.layers_mut() {
            let mut moved = IdVec::from_vec(vec![U32Id::from_u32(0); volume as usize]);

            for (&old_id, &new_id) in &new_ids {
                moved[new_id.to_usize_id()] = samples[old_id.to_usize_id()];
            }

            *samples = moved;
        }

        self.bounds = bounds;
        self.liveness = liveness;
        Ok(new_ids)
    }

    /// Rebuilds the grid at `bounds`. Each new cell draws from the old cell
    /// `source` picks for it: a live source makes the cell live with the
    /// source's samples, and a dead source or `None` leaves it empty. Several
    /// cells may draw from one source. Layers and `origin` stay. Returns the
    /// ids the old grid had live. Errors, changing nothing, if:
    ///
    /// 1. the grid would exceed [`MAX_GRID_CELLS`](Self::MAX_GRID_CELLS)
    /// 2. `source` picks a cell outside the old grid
    pub fn resample_voxels(
        &mut self,
        bounds: TyVector3U32,
        source: impl Fn(TyVector3U32) -> Option<TyVector3U32>,
    ) -> Result<Vec<U32Id<BVoxVoxel>>> {
        let volume = Self::volume_of(bounds);
        if volume > Self::MAX_GRID_CELLS {
            return Err(Error::GridCellCap { cells: volume });
        }

        // Per live new cell, the live old cell it copies.
        let mut liveness = VoxLiveness::new(volume as usize);
        let mut sources = Vec::new();
        for new_id in IdRange::from_len(volume as usize) {
            let position =
                Self::raster_position(bounds, new_id).expect("a raster index is within the grid");
            let Some(old_position) = source(position) else {
                continue;
            };
            let Some(old_id) = Self::raster_id(self.bounds, old_position) else {
                return Err(Error::ResampleSourceOutsideGrid {
                    position: old_position,
                    bounds: self.bounds,
                });
            };
            if self.liveness.is_live(old_id) {
                liveness.set_live(new_id, true);
                sources.push((new_id, old_id));
            }
        }

        // Non-live cells are ignored filler, so material 0 stands in.
        for (_, (_, samples)) in self.layers_mut() {
            let mut drawn = IdVec::from_vec(vec![U32Id::from_u32(0); volume as usize]);

            for &(new_id, old_id) in &sources {
                drawn[new_id.to_usize_id()] = samples[old_id.to_usize_id()];
            }

            *samples = drawn;
        }

        let old_ids = self.liveness.iter_live().collect();
        self.bounds = bounds;
        self.liveness = liveness;
        Ok(old_ids)
    }

    /// This object turned from Z-up to Y-up axes, `+z` to `+y` and `+y` to
    /// `-z`. The origin moves with the box, so the object keeps its place
    /// under its node.
    pub fn zup_to_yup(&self) -> Self {
        let TyVector3U32 { x, y, z } = self.bounds;
        self.permuted(
            TyVector3U32::new(x, z, y),
            TyVector3I32::new(self.origin.x, self.origin.z, -(self.origin.y + y as i32)),
            |cell| TyVector3U32::new(cell.x, cell.z, y - 1 - cell.y),
        )
    }

    /// This object turned from Y-up to Z-up axes, the inverse of
    /// [`zup_to_yup`](Self::zup_to_yup).
    pub fn yup_to_zup(&self) -> Self {
        let TyVector3U32 { x, y, z } = self.bounds;
        self.permuted(
            TyVector3U32::new(x, z, y),
            TyVector3I32::new(self.origin.x, -(self.origin.z + z as i32), self.origin.y),
            |cell| TyVector3U32::new(cell.x, z - 1 - cell.z, cell.y),
        )
    }

    /// A copy on the grid `bounds` at `origin`, each cell moved where `cell`
    /// sends it. `cell` maps this grid onto the new one, one to one.
    fn permuted(
        &self,
        bounds: TyVector3U32,
        origin: TyVector3I32,
        cell: impl Fn(TyVector3U32) -> TyVector3U32,
    ) -> Self {
        let mut copy = Self::new(self.name.clone(), bounds).expect("the grid keeps its cell count");

        copy.origin = origin;

        // Per cell of the new grid, the cell of this grid that moves there.
        let volume = self.liveness.len();
        let voxel_ids = IdRange::<U32Id<BVoxVoxel>>::from_len(volume);
        let mut source: IdVec<BVoxVoxel, U32Id<BVoxVoxel>> =
            IdVec::from_vec(vec![U32Id::from_u32(0); volume]);
        for voxel_id in voxel_ids.clone() {
            let position = self
                .voxel_position(voxel_id)
                .expect("a raster index is within the grid");
            let moved_id = copy
                .voxel_id(cell(position))
                .expect("the cell map lands inside the new grid");
            source[moved_id.to_usize_id()] = voxel_id;
        }

        for moved_id in voxel_ids.clone() {
            if self.liveness.is_live(source[moved_id.to_usize_id()]) {
                copy.liveness.set_live(moved_id, true);
            }
        }

        let mut samples = IdField::new();

        for (layer_id, (_, layer_samples)) in self.layers() {
            let moved: Vec<_> = voxel_ids
                .clone()
                .map(|moved_id| layer_samples[source[moved_id.to_usize_id()].to_usize_id()])
                .collect();

            samples.retain(layer_id, IdVec::from_vec(moved));
        }

        // The layers keep their ids, so the copy takes the whole layer id pool.
        copy.layer_ids = self.layer_ids.clone();

        copy.layer_palette_ids = self.layer_palette_ids.clone();

        copy.samples = samples;

        copy
    }

    /// The layers, as `(palette id, samples)` rows.
    fn layers(&self) -> IdStructView<'_, BVoxLayer, (&LayerPaletteIdColumn, &LayerSampleColumn)> {
        // Safety: both layer columns hold a value for every layer id.
        unsafe {
            self.layer_ids
                .view((&self.layer_palette_ids, &self.samples))
        }
    }

    /// The layers, as writable `(palette id, samples)` rows. Rows can also be
    /// added and removed.
    fn layers_mut(
        &mut self,
    ) -> IdStructViewMut<'_, BVoxLayer, (&mut LayerPaletteIdColumn, &mut LayerSampleColumn)> {
        // Safety: both layer columns hold a value for every layer id, and they
        // are the layer id pool's only columns.
        unsafe {
            self.layer_ids
                .view_mut((&mut self.layer_palette_ids, &mut self.samples))
        }
    }
}

impl Clone for VoxObject {
    fn clone(&self) -> Self {
        let mut samples = IdField::new();

        for (layer_id, (_, layer_samples)) in self.layers() {
            samples.retain(layer_id, layer_samples.clone());
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
}

impl Drop for VoxObject {
    fn drop(&mut self) {
        self.layers_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use crate::{BVoxLayer, BVoxMaterial, BVoxPalette, Error, VoxObject};
    use branded_id::U32Id;
    use std::collections::HashMap;
    use ty_math::{TyVector3I32, TyVector3U32};

    fn material_id(index: u32) -> U32Id<BVoxMaterial> {
        U32Id::from_u32(index)
    }

    /// A `1 x 2 x 3` grid seated at `(5, 6, 7)` with two layers, live at
    /// `(0, 1, 0)` sampling materials 3 and 4 and at `(0, 0, 2)` sampling 5
    /// and 6.
    fn seated_object() -> (VoxObject, [U32Id<BVoxLayer>; 2]) {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 2, 3)).unwrap();
        object.set_origin(TyVector3I32::new(5, 6, 7));
        let first_layer_id = object.retain_layer(U32Id::<BVoxPalette>::from_u32(0), material_id(0));
        let second_layer_id =
            object.retain_layer(U32Id::<BVoxPalette>::from_u32(1), material_id(1));
        let first_id = object.voxel_id(TyVector3U32::new(0, 1, 0)).unwrap();
        object
            .retain_voxel(first_id, &[material_id(3), material_id(4)])
            .unwrap();
        let second_id = object.voxel_id(TyVector3U32::new(0, 0, 2)).unwrap();
        object
            .retain_voxel(second_id, &[material_id(5), material_id(6)])
            .unwrap();

        (object, [first_layer_id, second_layer_id])
    }

    /// The `(position, samples)` of every live voxel, in raster order.
    fn live_cells(object: &VoxObject) -> Vec<(TyVector3U32, Vec<U32Id<BVoxMaterial>>)> {
        let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();
        object
            .iter_live()
            .map(|voxel_id| {
                let samples = layer_ids
                    .iter()
                    .map(|&layer_id| object.voxel_material(voxel_id, layer_id).unwrap())
                    .collect();
                (object.voxel_position(voxel_id).unwrap(), samples)
            })
            .collect()
    }

    #[test]
    fn zup_to_yup_turns_the_grid_about_x() {
        let (object, _) = seated_object();
        let turned = object.zup_to_yup();

        assert_eq!(turned.name(), "o");
        assert_eq!(turned.bounds(), TyVector3U32::new(1, 3, 2));
        // The box `[6, 8)` on the old `y` lands on `[-8, -6)` on the new `z`.
        assert_eq!(turned.origin(), TyVector3I32::new(5, 7, -8));
        let layers: Vec<_> = turned.iter_layers().collect();
        assert_eq!(layers, object.iter_layers().collect::<Vec<_>>());
        assert_eq!(
            live_cells(&turned),
            vec![
                (
                    TyVector3U32::new(0, 0, 0),
                    vec![material_id(3), material_id(4)]
                ),
                (
                    TyVector3U32::new(0, 2, 1),
                    vec![material_id(5), material_id(6)]
                ),
            ]
        );
    }

    #[test]
    fn remap_voxels_moves_live_voxels_and_their_samples() {
        let (mut object, _) = seated_object();

        // Swap y and z onto a `1 x 3 x 2` grid.
        let voxel_ids = object
            .remap_voxels(TyVector3U32::new(1, 3, 2), |p| {
                TyVector3I32::new(p.x as i32, p.z as i32, p.y as i32)
            })
            .unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(1, 3, 2));
        assert_eq!(object.origin(), TyVector3I32::new(5, 6, 7));
        assert_eq!(
            live_cells(&object),
            vec![
                (
                    TyVector3U32::new(0, 0, 1),
                    vec![material_id(3), material_id(4)]
                ),
                (
                    TyVector3U32::new(0, 2, 0),
                    vec![material_id(5), material_id(6)]
                ),
            ]
        );
        assert_eq!(
            voxel_ids,
            HashMap::from([
                (U32Id::from_u32(3), U32Id::from_u32(1)),
                (U32Id::from_u32(2), U32Id::from_u32(4)),
            ])
        );
    }

    #[test]
    fn remap_voxels_rejects_a_bad_move_without_changing_state() {
        let (mut object, _) = seated_object();
        let before = live_cells(&object);

        assert_eq!(
            object.remap_voxels(TyVector3U32::new(1, 2, 3), |p| p.as_ivec3()
                - TyVector3I32::Y),
            Err(Error::RemappedVoxelOutsideGrid {
                voxel_id: U32Id::from_u32(2),
                position: TyVector3I32::new(0, -1, 2),
                bounds: TyVector3U32::new(1, 2, 3),
            })
        );
        assert_eq!(
            object.remap_voxels(TyVector3U32::new(1, 1, 1), |_| TyVector3I32::ZERO),
            Err(Error::RemappedVoxelCollision {
                voxel_id: U32Id::from_u32(3),
                other_voxel_id: U32Id::from_u32(2),
            })
        );
        assert!(matches!(
            object.remap_voxels(TyVector3U32::splat(1 << 10), |p| p.as_ivec3()),
            Err(Error::GridCellCap { .. })
        ));

        assert_eq!(object.bounds(), TyVector3U32::new(1, 2, 3));
        assert_eq!(live_cells(&object), before);
    }

    #[test]
    fn resample_voxels_draws_each_cell_from_its_source() {
        let (mut object, _) = seated_object();

        // Double the grid along y, each old cell filling two new ones.
        let old_ids = object
            .resample_voxels(TyVector3U32::new(1, 4, 3), |p| {
                Some(TyVector3U32::new(p.x, p.y / 2, p.z))
            })
            .unwrap();

        assert_eq!(object.bounds(), TyVector3U32::new(1, 4, 3));
        assert_eq!(object.origin(), TyVector3I32::new(5, 6, 7));
        assert_eq!(
            live_cells(&object),
            vec![
                (
                    TyVector3U32::new(0, 0, 2),
                    vec![material_id(5), material_id(6)]
                ),
                (
                    TyVector3U32::new(0, 1, 2),
                    vec![material_id(5), material_id(6)]
                ),
                (
                    TyVector3U32::new(0, 2, 0),
                    vec![material_id(3), material_id(4)]
                ),
                (
                    TyVector3U32::new(0, 3, 0),
                    vec![material_id(3), material_id(4)]
                ),
            ]
        );
        assert_eq!(old_ids, [U32Id::from_u32(2), U32Id::from_u32(3)]);

        // A `None` source leaves its cell empty.
        object
            .resample_voxels(TyVector3U32::splat(1), |_| None)
            .unwrap();

        assert!(live_cells(&object).is_empty());
    }

    #[test]
    fn resample_voxels_rejects_a_bad_source_without_changing_state() {
        let (mut object, _) = seated_object();
        let before = live_cells(&object);

        assert_eq!(
            object.resample_voxels(TyVector3U32::new(1, 2, 3), |p| Some(p + TyVector3U32::Z)),
            Err(Error::ResampleSourceOutsideGrid {
                position: TyVector3U32::new(0, 0, 3),
                bounds: TyVector3U32::new(1, 2, 3),
            })
        );
        assert!(matches!(
            object.resample_voxels(TyVector3U32::splat(1 << 10), Some),
            Err(Error::GridCellCap { .. })
        ));

        assert_eq!(object.bounds(), TyVector3U32::new(1, 2, 3));
        assert_eq!(live_cells(&object), before);
    }

    #[test]
    fn the_axis_turns_are_inverses() {
        let (object, _) = seated_object();
        for restored in [
            object.zup_to_yup().yup_to_zup(),
            object.yup_to_zup().zup_to_yup(),
        ] {
            assert_eq!(restored.bounds(), object.bounds());
            assert_eq!(restored.origin(), object.origin());
            assert_eq!(live_cells(&restored), live_cells(&object));
        }
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

        assert_eq!(
            object
                .iter_layers()
                .map(|(layer_id, _)| layer_id)
                .collect::<Vec<_>>(),
            [first_id, second_id, third_id]
        );

        assert_eq!(object.move_layer(third_id, 0), Ok(()));
        assert_eq!(
            object
                .iter_layers()
                .map(|(layer_id, _)| layer_id)
                .collect::<Vec<_>>(),
            [third_id, first_id, second_id]
        );

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

    #[test]
    fn a_clone_copies_every_layer_and_voxel() {
        let (mut object, [first_layer_id, _]) = seated_object();

        object.release_layer(first_layer_id).unwrap();

        let mut copy = object.clone();

        assert_eq!(copy.name(), "o");
        assert_eq!(copy.bounds(), object.bounds());
        assert_eq!(copy.origin(), object.origin());
        assert_eq!(
            copy.iter_layers().collect::<Vec<_>>(),
            object.iter_layers().collect::<Vec<_>>()
        );
        assert_eq!(live_cells(&copy), live_cells(&object));

        let voxel_id = copy.iter_live().next().unwrap();
        copy.release_voxel(voxel_id).unwrap();

        assert_eq!(object.live_count(), 2);
    }

    #[test]
    fn relabel_layer_palettes_keeps_the_samples() {
        let (mut object, _) = seated_object();

        let before = live_cells(&object);

        object.relabel_layer_palettes(|palette_id| U32Id::from_u32(palette_id.to_u32() + 10));

        let palette_ids: Vec<_> = object
            .iter_layers()
            .map(|(_, palette_id)| palette_id.to_u32())
            .collect();

        assert_eq!(palette_ids, [10, 11]);
        assert_eq!(live_cells(&object), before);
    }

    #[test]
    fn raster_ids_number_any_grid_within_the_cap() {
        let bounds = TyVector3U32::new(2, 3, 4);
        let position = TyVector3U32::new(1, 2, 3);
        let id = VoxObject::raster_id(bounds, position).unwrap();

        assert_eq!(id.to_u32(), 12 + 2 * 4 + 3);
        assert_eq!(VoxObject::raster_position(bounds, id), Some(position));
        assert_eq!(
            VoxObject::raster_id(bounds, TyVector3U32::new(2, 0, 0)),
            None
        );
        assert_eq!(
            VoxObject::raster_position(bounds, U32Id::from_u32(24)),
            None
        );

        let object = VoxObject::new("o".to_owned(), bounds).unwrap();
        assert_eq!(object.voxel_id(position), Some(id));

        // Past the cap, no grid of these bounds exists to number.
        let over = TyVector3U32::new(1 << 16, 1 << 16, 1);
        assert_eq!(VoxObject::raster_id(over, TyVector3U32::new(1, 1, 0)), None);
        assert_eq!(VoxObject::raster_position(over, U32Id::from_u32(1)), None);
    }
}
