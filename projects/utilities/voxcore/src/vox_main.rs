use crate::{
    BVoxEffectiveProperty, BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette,
    BVoxProperty, BVoxValuePool, BVoxValuePoolValue, BVoxVoxel, Error, Result, VoxEffectivePalette,
    VoxEffectiveProperty, VoxGcRemap, VoxHierarchyNode, VoxObject, VoxPalette, VoxRuntimeState,
    VoxValue, VoxValuePool, VoxValuePoolFlaw,
};
use branded_id::{IdVec, U32Id, UsizeId, soa::IdRemap};
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionExt, TyVector3F64, TyVector3I32, UNIT_ROTATION_TOLERANCE};

/// The in-memory state of a voxel model: its objects, shared palettes, scene
/// hierarchy, and roots.
///
/// Ids are meaningful only within this state. Every mutation checks the
/// cross-references it could break, so a state reached through the public API
/// never violates a referential rule; [`validate`](Self::validate) audits
/// them.
#[derive(Debug, Default)]
pub struct VoxMain {
    /// The runtime scene: objects.
    runtime_state: VoxRuntimeState,

    /// Optional user-extension namespace; the core format assigns it no
    /// meaning.
    ext: Option<VoxValue>,
}

impl VoxMain {
    /// Adds an object at the end of the listing, returning its id. Errors,
    /// changing nothing, if a layer references a palette that is not one of
    /// this state's or a live voxel samples a material that is not one of its
    /// layer's palette's.
    pub fn add_object(&mut self, object: VoxObject) -> Result<U32Id<BVoxObject>> {
        for (layer_id, palette_id) in object.iter_layers() {
            let Some(palette) = self.palette(palette_id) else {
                return Err(Error::LayerPaletteRef {
                    layer_id,
                    palette_id,
                });
            };
            let samples = object
                .iter_live_samples(layer_id)
                .expect("an iterated layer is one of the object's layers");
            for (voxel_id, material_id) in samples {
                if !palette.contains_material(material_id) {
                    return Err(Error::LayerSampleMaterial {
                        layer_id,
                        voxel_id,
                        material_id,
                    });
                }
            }
        }
        let id = self.runtime_state.object_ids.retain();
        self.runtime_state.objects.retain(id, object);
        Ok(id)
    }

    /// Number of objects.
    pub fn object_count(&self) -> usize {
        self.runtime_state.object_ids.len()
    }

    /// The object `id`, or `None` if not one of this state's.
    pub fn object(&self, id: U32Id<BVoxObject>) -> Option<&VoxObject> {
        // Safety: retained ids have a value.
        self.runtime_state
            .object_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.objects.get(id) })
    }

    /// Objects in listing order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BVoxObject>, &VoxObject)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .object_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.objects.get(id) }))
    }

    /// Adds a layer referencing `palette_id` to object `object_id`, after its
    /// existing layers, back-filling every voxel with `default_material_id`
    /// and returning the layer's id. Errors, changing nothing, if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. `palette_id` is not one of this state's
    /// 3. `default_material_id` is not one of `palette_id`'s materials
    pub fn add_layer(
        &mut self,
        object_id: U32Id<BVoxObject>,
        palette_id: U32Id<BVoxPalette>,
        default_material_id: U32Id<BVoxMaterial>,
    ) -> Result<U32Id<BVoxLayer>> {
        if !self.runtime_state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }
        let Some(palette_ref) = self.palette(palette_id) else {
            return Err(Error::UnknownPalette { palette_id });
        };
        if !palette_ref.contains_material(default_material_id) {
            return Err(Error::UnknownMaterial {
                material_id: default_material_id,
            });
        }
        // Safety: the object id is retained.
        Ok(unsafe { self.runtime_state.objects.get_mut(object_id) }
            .add_layer(palette_id, default_material_id))
    }

    /// Removes layer `layer_id` from object `object_id`, dropping its
    /// per-voxel sample column. Errors, changing nothing, if `object_id` is
    /// not one of this state's or `layer_id` is not one of the object's.
    pub fn remove_layer(
        &mut self,
        object_id: U32Id<BVoxObject>,
        layer_id: U32Id<BVoxLayer>,
    ) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }
        // Safety: the object id is retained.
        unsafe { self.runtime_state.objects.get_mut(object_id) }.remove_layer(layer_id)
    }

    /// Moves layer `layer_id` of object `object_id` to position `index` in
    /// its layer order. Errors, changing nothing, if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. `layer_id` is not one of the object's
    /// 3. `index` is at or past its layer count
    pub fn move_layer(
        &mut self,
        object_id: U32Id<BVoxObject>,
        layer_id: U32Id<BVoxLayer>,
        index: usize,
    ) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }
        // Safety: the object id is retained.
        unsafe { self.runtime_state.objects.get_mut(object_id) }.move_layer(layer_id, index)
    }

    /// Makes the voxel at `voxel_id` in object `object_id` live with one
    /// `sample_ids` material per layer, in layer order. Errors, changing
    /// nothing, if:
    ///
    /// 1. `object_id` is not one of this state's
    /// 2. `voxel_id` is outside its grid
    /// 3. `sample_ids` has the wrong length
    /// 4. a sample is not one of its layer's palette's materials
    pub fn retain_voxel(
        &mut self,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
        sample_ids: &[U32Id<BVoxMaterial>],
    ) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }
        // Safety: the object id is retained.
        let object_ref = unsafe { self.runtime_state.objects.get(object_id) };
        if object_ref.voxel_position(voxel_id).is_none() {
            return Err(Error::UnknownVoxel { voxel_id });
        }
        if sample_ids.len() != object_ref.layer_count() {
            return Err(Error::SampleArity {
                samples: sample_ids.len(),
                layers: object_ref.layer_count(),
            });
        }
        for ((layer_id, palette_id), &material_id) in object_ref.iter_layers().zip(sample_ids) {
            let palette = self
                .palette(palette_id)
                .expect("a layer references a live palette");
            if !palette.contains_material(material_id) {
                return Err(Error::LayerSampleMaterial {
                    layer_id,
                    voxel_id,
                    material_id,
                });
            }
        }
        // Safety: the object id is retained; the grid and arity were checked.
        unsafe { self.runtime_state.objects.get_mut(object_id) }.retain_voxel(voxel_id, sample_ids)
    }

    /// Makes the voxel at `voxel_id` in object `object_id` empty, leaving its
    /// samples in place but ignored. Errors, changing nothing, if `object_id`
    /// is not one of this state's or `voxel_id` is outside its grid.
    pub fn release_voxel(
        &mut self,
        object_id: U32Id<BVoxObject>,
        voxel_id: U32Id<BVoxVoxel>,
    ) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }
        // Safety: the object id is retained.
        unsafe { self.runtime_state.objects.get_mut(object_id) }.release_voxel(voxel_id)
    }

    /// Sets the grid origin of object `object_id`. Errors, changing nothing,
    /// if `object_id` is not one of this state's.
    pub fn set_object_origin(
        &mut self,
        object_id: U32Id<BVoxObject>,
        origin: TyVector3I32,
    ) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(object_id) {
            return Err(Error::UnknownObject { object_id });
        }
        // Safety: the object id is retained.
        unsafe { self.runtime_state.objects.get_mut(object_id) }.set_origin(origin);
        Ok(())
    }

    /// Moves object `id` to position `index` in the listing, shifting the
    /// objects between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's objects or `index` is at or
    /// past [`object_count`](Self::object_count).
    pub fn move_object(&mut self, id: U32Id<BVoxObject>, index: usize) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(id) {
            return Err(Error::UnknownObject { object_id: id });
        }
        let count = self.runtime_state.object_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.runtime_state.object_ids.move_to(id, index);
        Ok(())
    }

    /// The listing position of object `id`, or `None` if `id` is not one of
    /// this state's objects.
    pub fn object_index(&self, id: U32Id<BVoxObject>) -> Option<usize> {
        self.runtime_state.object_ids.index_of(id)
    }

    /// Adds a shared palette at the end of the listing, returning its id.
    /// Errors, changing nothing, if:
    ///
    /// 1. the palette holds no materials
    /// 2. a property names a value pool that is not one of this state's
    /// 3. a material draws a value that is not one of its property's pool's
    pub fn add_palette(&mut self, palette: VoxPalette) -> Result<U32Id<BVoxPalette>> {
        if palette.material_count() == 0 {
            return Err(Error::NoPaletteMaterials);
        }
        for (property_id, property) in palette.iter_properties() {
            let Some(value_pool) = self.value_pool(property.value_pool_id) else {
                return Err(Error::PropertyValuePoolRef {
                    property_id,
                    value_pool_id: property.value_pool_id,
                });
            };
            for material_id in palette.iter_materials() {
                let value_id = palette
                    .value_id(material_id, property_id)
                    .expect("a material has a value id for every property");
                if !value_pool.contains_value(value_id) {
                    return Err(Error::MaterialValueRef {
                        property_id,
                        material_id,
                    });
                }
            }
        }
        let id = self.runtime_state.palette_ids.retain();
        self.runtime_state.palettes.retain(id, palette);
        Ok(id)
    }

    /// Number of shared palettes.
    pub fn palette_count(&self) -> usize {
        self.runtime_state.palette_ids.len()
    }

    /// The palette `id`, or `None` if not one of this state's.
    pub fn palette(&self, id: U32Id<BVoxPalette>) -> Option<&VoxPalette> {
        // Safety: retained ids have a value.
        self.runtime_state
            .palette_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.palettes.get(id) })
    }

    /// Palettes in listing order, as `(id, palette)`.
    pub fn iter_palettes(&self) -> impl Iterator<Item = (U32Id<BVoxPalette>, &VoxPalette)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .palette_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.palettes.get(id) }))
    }

    /// Moves palette `id` to position `index` in the listing, shifting the
    /// palettes between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's palettes or `index` is at
    /// or past [`palette_count`](Self::palette_count).
    pub fn move_palette(&mut self, id: U32Id<BVoxPalette>, index: usize) -> Result<()> {
        if !self.runtime_state.palette_ids.is_retained(id) {
            return Err(Error::UnknownPalette { palette_id: id });
        }
        let count = self.runtime_state.palette_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.runtime_state.palette_ids.move_to(id, index);
        Ok(())
    }

    /// The listing position of palette `id`, or `None` if `id` is not one of
    /// this state's palettes.
    pub fn palette_index(&self, id: U32Id<BVoxPalette>) -> Option<usize> {
        self.runtime_state.palette_ids.index_of(id)
    }

    /// Adds a property named `name` on `value_pool_id` to palette
    /// `palette_id`, back-filling its existing materials with
    /// `default_value_id`, and returns the property's id. Errors, changing
    /// nothing, if:
    ///
    /// 1. `palette_id` is not one of this state's
    /// 2. `value_pool_id` is not one of this state's
    /// 3. `default_value_id` is not one of `value_pool_id`'s values
    /// 4. the palette already has a property named `name`
    pub fn add_property(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        name: String,
        value_pool_id: U32Id<BVoxValuePool>,
        default_value_id: U32Id<BVoxValuePoolValue>,
    ) -> Result<U32Id<BVoxProperty>> {
        if !self.runtime_state.palette_ids.is_retained(palette_id) {
            return Err(Error::UnknownPalette { palette_id });
        }
        let Some(value_pool) = self.value_pool(value_pool_id) else {
            return Err(Error::UnknownValuePool { value_pool_id });
        };
        if !value_pool.contains_value(default_value_id) {
            return Err(Error::UnknownValuePoolValue {
                value_id: default_value_id,
            });
        }
        // Safety: the palette id is retained.
        unsafe { self.runtime_state.palettes.get_mut(palette_id) }.add_property(
            name,
            value_pool_id,
            default_value_id,
        )
    }

    /// Adds a material with one value id per property, in property order, to
    /// palette `palette_id` and returns its id. Errors, changing nothing, if:
    ///
    /// 1. `palette_id` is not one of this state's
    /// 2. `value_ids` has the wrong length
    /// 3. a value id is not one of its property's pool's
    pub fn add_material(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        value_ids: Vec<U32Id<BVoxValuePoolValue>>,
    ) -> Result<U32Id<BVoxMaterial>> {
        if !self.runtime_state.palette_ids.is_retained(palette_id) {
            return Err(Error::UnknownPalette { palette_id });
        }
        // Safety: the palette id is retained.
        let palette_ref = unsafe { self.runtime_state.palettes.get(palette_id) };
        if value_ids.len() != palette_ref.property_count() {
            return Err(Error::MaterialValueArity {
                values: value_ids.len(),
                properties: palette_ref.property_count(),
            });
        }
        for ((_, property), &value_id) in palette_ref.iter_properties().zip(&value_ids) {
            let value_pool = self
                .value_pool(property.value_pool_id)
                .expect("a property names a live value pool");
            if !value_pool.contains_value(value_id) {
                return Err(Error::UnknownValuePoolValue { value_id });
            }
        }
        // Safety: the palette id is retained; the arity was checked.
        unsafe { self.runtime_state.palettes.get_mut(palette_id) }.add_material(value_ids)
    }

    /// Removes property `property_id` from palette `palette_id`. Errors,
    /// changing nothing, if `palette_id` is not one of this state's or
    /// `property_id` is not one of the palette's.
    pub fn remove_property(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        property_id: U32Id<BVoxProperty>,
    ) -> Result<()> {
        if !self.runtime_state.palette_ids.is_retained(palette_id) {
            return Err(Error::UnknownPalette { palette_id });
        }
        // Safety: the palette id is retained.
        unsafe { self.runtime_state.palettes.get_mut(palette_id) }.remove_property(property_id)
    }

    /// Adds a shared value pool at the end of the listing, returning its id.
    pub fn add_value_pool(&mut self, value_pool: VoxValuePool) -> U32Id<BVoxValuePool> {
        let id = self.runtime_state.value_pool_ids.retain();
        self.runtime_state.value_pools.retain(id, value_pool);
        id
    }

    /// Number of shared value pools.
    pub fn value_pool_count(&self) -> usize {
        self.runtime_state.value_pool_ids.len()
    }

    /// The value pool `id`, or `None` if not one of this state's.
    pub fn value_pool(&self, id: U32Id<BVoxValuePool>) -> Option<&VoxValuePool> {
        // Safety: retained ids have a value.
        self.runtime_state
            .value_pool_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.value_pools.get(id) })
    }

    /// Value pools in listing order, as `(id, pool)`.
    pub fn iter_value_pools(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxValuePool>, &VoxValuePool)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .value_pool_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.value_pools.get(id) }))
    }

    /// Moves value pool `id` to position `index` in the listing, shifting the
    /// pools between its old and new positions one slot. Errors, changing
    /// nothing, if `id` is not one of this state's pools or `index` is at or
    /// past [`value_pool_count`](Self::value_pool_count).
    pub fn move_value_pool(&mut self, id: U32Id<BVoxValuePool>, index: usize) -> Result<()> {
        if !self.runtime_state.value_pool_ids.is_retained(id) {
            return Err(Error::UnknownValuePool { value_pool_id: id });
        }
        let count = self.runtime_state.value_pool_ids.len();
        if index >= count {
            return Err(Error::IndexPastCount { index, count });
        }
        self.runtime_state.value_pool_ids.move_to(id, index);
        Ok(())
    }

    /// The listing position of value pool `id`, or `None` if `id` is not one
    /// of this state's pools.
    pub fn value_pool_index(&self, id: U32Id<BVoxValuePool>) -> Option<usize> {
        self.runtime_state.value_pool_ids.index_of(id)
    }

    /// Resolves what `material_id` in `palette_id` draws for `property_id`:
    /// the value pool the property draws from and the value id in that pool.
    /// `None` if any id is not this state's, `property_id` is not
    /// `palette_id`'s, or the property names a pool this state does not hold.
    /// Read the value at that id out of the returned pool by the pool's
    /// kind.
    pub fn material_value(
        &self,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
        property_id: U32Id<BVoxProperty>,
    ) -> Option<(&VoxValuePool, U32Id<BVoxValuePoolValue>)> {
        let palette = self.palette(palette_id)?;
        let value_id = palette.value_id(material_id, property_id)?;
        let value_pool = self.value_pool(palette.property(property_id)?.value_pool_id)?;
        Some((value_pool, value_id))
    }

    /// The effective palette of `object`, resolving its layer override rule
    /// once. Layers are walked front to back, each palette property landing
    /// at its name's entry, so the last supplying layer wins while the first
    /// fixes the entry's position. Errors if a layer references a palette
    /// that is not one of this state's.
    pub fn effective_palette<'a>(
        &'a self,
        object: &'a VoxObject,
    ) -> Result<VoxEffectivePalette<'a>> {
        let mut properties: IdVec<BVoxEffectiveProperty, VoxEffectiveProperty<'a>> =
            IdVec::default();
        let mut property_id_by_name: HashMap<&'a str, UsizeId<BVoxEffectiveProperty>> =
            HashMap::new();

        for (layer_id, palette_id) in object.iter_layers() {
            let Some(palette) = self.palette(palette_id) else {
                return Err(Error::LayerPaletteRef {
                    layer_id,
                    palette_id,
                });
            };

            for (property_id, property) in palette.iter_properties() {
                let value_pool = self
                    .value_pool(property.value_pool_id)
                    .expect("a property names a live value pool");

                let entry = VoxEffectiveProperty {
                    name: property.name.as_str(),
                    layer_id,
                    palette_id,
                    palette,
                    property_id,
                    value_pool,
                };

                match property_id_by_name.get(property.name.as_str()) {
                    Some(&effective_id) => properties[effective_id] = entry,
                    None => {
                        let effective_id = properties.push(entry);
                        property_id_by_name.insert(property.name.as_str(), effective_id);
                    }
                }
            }
        }

        Ok(VoxEffectivePalette {
            object,
            properties,
            property_id_by_name,
        })
    }

    /// Adds a hierarchy node at the end of the listing, returning its id. The
    /// node's id is fresh to every existing child list, so a node whose
    /// children are already live can never close a cycle. For a batch whose
    /// nodes reference each other, use
    /// [`add_hierarchy_nodes`](Self::add_hierarchy_nodes). Errors, changing
    /// nothing, if:
    ///
    /// 1. a child node or child object is not one of this state's
    /// 2. a child repeats
    /// 3. the transform is malformed
    pub fn add_hierarchy_node(
        &mut self,
        node: VoxHierarchyNode,
    ) -> Result<U32Id<BVoxHierarchyNode>> {
        self.check_inserted_node(&node, 0, &HashSet::new())?;
        let id = self.runtime_state.hierarchy_node_ids.retain();
        self.runtime_state.hierarchy_nodes.retain(id, node);
        Ok(id)
    }

    /// Adds a batch of hierarchy nodes at the end of the listing, assigning
    /// ids in listing order and returning them. A node's children may
    /// reference any already-live node or any node in the batch by the id it
    /// will take, so a listing with forward references loads in one call.
    /// Errors, changing nothing, if:
    ///
    /// 1. a child resolves to neither
    /// 2. a child repeats within a node
    /// 3. a transform is malformed
    /// 4. the batch's `child_node_ids` edges form a cycle
    pub fn add_hierarchy_nodes(
        &mut self,
        nodes: Vec<VoxHierarchyNode>,
    ) -> Result<Vec<U32Id<BVoxHierarchyNode>>> {
        // The ids the batch will take, named before any of it is inserted so
        // every check runs before any mutation.
        let prospective_ids: Vec<U32Id<BVoxHierarchyNode>> = (0..nodes.len())
            .map(|index| self.runtime_state.hierarchy_node_ids.peek_nth(index))
            .collect();

        let batch_ids: HashSet<U32Id<BVoxHierarchyNode>> =
            prospective_ids.iter().copied().collect();
        for (index, node) in nodes.iter().enumerate() {
            self.check_inserted_node(node, index, &batch_ids)?;
        }

        // An edge leaving the batch lands on an already-live node, whose
        // children are frozen and reference only other live nodes, so it can
        // never lead back in. Only the batch-internal edges can cycle.
        let index_of: HashMap<U32Id<BVoxHierarchyNode>, usize> = prospective_ids
            .iter()
            .enumerate()
            .map(|(index, &id)| (id, index))
            .collect();
        let children: Vec<&[U32Id<BVoxHierarchyNode>]> = nodes
            .iter()
            .map(|node| node.child_node_ids.as_slice())
            .collect();
        if let Some(index) = first_cycle_position(&children, &index_of) {
            return Err(Error::InsertedCycle { index });
        }

        let ids: Vec<U32Id<BVoxHierarchyNode>> = nodes
            .into_iter()
            .map(|node| {
                let id = self.runtime_state.hierarchy_node_ids.retain();
                self.runtime_state.hierarchy_nodes.retain(id, node);
                id
            })
            .collect();
        debug_assert_eq!(ids, prospective_ids, "the pool assigned the predicted ids");
        Ok(ids)
    }

    /// Checks a node about to be inserted at listing position `index` of its
    /// batch: every child node resolves against this state or the batch's
    /// prospective ids, every child object is live, no child repeats, and the
    /// transform is finite with a non-zero scale and a unit rotation.
    fn check_inserted_node(
        &self,
        node: &VoxHierarchyNode,
        index: usize,
        batch_ids: &HashSet<U32Id<BVoxHierarchyNode>>,
    ) -> Result<()> {
        let mut seen_child_node_ids = HashSet::with_capacity(node.child_node_ids.len());
        for &child_id in &node.child_node_ids {
            if self.hierarchy_node(child_id).is_none() && !batch_ids.contains(&child_id) {
                return Err(Error::UnknownHierarchyNode { node_id: child_id });
            }
            if !seen_child_node_ids.insert(child_id) {
                return Err(Error::InsertedDuplicateChildNode { index, child_id });
            }
        }
        let mut seen_child_object_ids = HashSet::with_capacity(node.child_object_ids.len());
        for &object_id in &node.child_object_ids {
            if self.object(object_id).is_none() {
                return Err(Error::UnknownObject { object_id });
            }
            if !seen_child_object_ids.insert(object_id) {
                return Err(Error::InsertedDuplicateChildObject { index, object_id });
            }
        }

        // The rotation needs no finiteness guard of its own: a non-finite
        // component fails the unit-length check.
        let position = node.transform.position;
        let scale = node.transform.scale;
        if !vector_is_finite(position) || !vector_is_finite(scale) {
            return Err(Error::InsertedNonFiniteTransform { index });
        }
        if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
            return Err(Error::InsertedZeroScale { index });
        }
        if !node
            .transform
            .rotation
            .is_normalized_within(UNIT_ROTATION_TOLERANCE)
        {
            return Err(Error::InsertedNonUnitRotation { index });
        }
        Ok(())
    }

    /// Number of hierarchy nodes.
    pub fn hierarchy_node_count(&self) -> usize {
        self.runtime_state.hierarchy_node_ids.len()
    }

    /// The hierarchy node `id`, or `None` if not one of this state's.
    pub fn hierarchy_node(&self, id: U32Id<BVoxHierarchyNode>) -> Option<&VoxHierarchyNode> {
        // Safety: retained ids have a value.
        self.runtime_state
            .hierarchy_node_ids
            .is_retained(id)
            .then(|| unsafe { self.runtime_state.hierarchy_nodes.get(id) })
    }

    /// Hierarchy nodes in listing order, as `(id, node)`.
    pub fn iter_hierarchy_nodes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxHierarchyNode>, &VoxHierarchyNode)> + '_ {
        // Safety: retained ids have a value.
        self.runtime_state
            .hierarchy_node_ids
            .iter()
            .map(move |id| (id, unsafe { self.runtime_state.hierarchy_nodes.get(id) }))
    }

    /// The scene's roots: hierarchy node ids.
    pub fn root_hierarchy_node_ids(&self) -> &[U32Id<BVoxHierarchyNode>] {
        &self.runtime_state.root_hierarchy_node_ids
    }

    /// Replaces the scene's roots. Errors, changing nothing, if a root is not
    /// one of this state's nodes or repeats.
    pub fn set_root_hierarchy_node_ids(
        &mut self,
        root_ids: Vec<U32Id<BVoxHierarchyNode>>,
    ) -> Result<()> {
        let mut seen_ids = HashSet::with_capacity(root_ids.len());
        for &root_id in &root_ids {
            if self.hierarchy_node(root_id).is_none() {
                return Err(Error::Root { root_id });
            }
            if !seen_ids.insert(root_id) {
                return Err(Error::DuplicateRoot { root_id });
            }
        }
        self.runtime_state.root_hierarchy_node_ids = root_ids;
        Ok(())
    }

    /// Appends a root. Errors, changing nothing, if `root_id` is not one of
    /// this state's nodes or is already a root.
    pub fn push_root_hierarchy_node_id(&mut self, root_id: U32Id<BVoxHierarchyNode>) -> Result<()> {
        if self.hierarchy_node(root_id).is_none() {
            return Err(Error::Root { root_id });
        }
        if self
            .runtime_state
            .root_hierarchy_node_ids
            .contains(&root_id)
        {
            return Err(Error::DuplicateRoot { root_id });
        }
        self.runtime_state.root_hierarchy_node_ids.push(root_id);
        Ok(())
    }

    /// The user-extension value, or `None` if unset.
    pub fn ext(&self) -> Option<&VoxValue> {
        self.ext.as_ref()
    }

    /// Sets or clears the user-extension value.
    pub fn set_ext(&mut self, ext: Option<VoxValue>) {
        self.ext = ext;
    }

    /// Removes object `id`, detaching it from every node's `child_object_ids`.
    /// Errors, changing nothing, if `id` is not one of this state's objects.
    /// Leaves a hole until [`gc`](Self::gc) renumbers for a deterministic
    /// save.
    pub fn remove_object(&mut self, id: U32Id<BVoxObject>) -> Result<()> {
        if !self.runtime_state.object_ids.is_retained(id) {
            return Err(Error::UnknownObject { object_id: id });
        }
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.runtime_state.hierarchy_nodes.get_mut(node_id) };
            node.child_object_ids.retain(|&object_id| object_id != id);
        }
        // Safety: a retained object id has a value.
        unsafe { self.runtime_state.objects.release(id) };
        self.runtime_state.object_ids.release_stable(id);
        Ok(())
    }

    /// Removes palette `id`, detaching every object reference to it (along with
    /// that reference's per-voxel sample column). Errors, changing nothing, if
    /// `id` is not one of this state's palettes. Leaves a hole until
    /// [`gc`](Self::gc) renumbers.
    pub fn remove_palette(&mut self, id: U32Id<BVoxPalette>) -> Result<()> {
        if !self.runtime_state.palette_ids.is_retained(id) {
            return Err(Error::UnknownPalette { palette_id: id });
        }
        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.runtime_state.objects.get_mut(object_id) };
            object.remove_layers_to(id);
        }
        // Safety: a retained palette id has a value; its Drop frees its cells.
        unsafe { self.runtime_state.palettes.release(id) };
        self.runtime_state.palette_ids.release_stable(id);
        Ok(())
    }

    /// Removes hierarchy node `id`, detaching it from every `child_node_ids` list
    /// and from the roots. Errors, changing nothing, if `id` is not one of
    /// this state's nodes. Leaves a hole until [`gc`](Self::gc) renumbers.
    pub fn remove_hierarchy_node(&mut self, id: U32Id<BVoxHierarchyNode>) -> Result<()> {
        if !self.runtime_state.hierarchy_node_ids.is_retained(id) {
            return Err(Error::UnknownHierarchyNode { node_id: id });
        }
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.runtime_state.hierarchy_nodes.get_mut(node_id) };
            node.child_node_ids.retain(|&child_id| child_id != id);
        }
        self.runtime_state
            .root_hierarchy_node_ids
            .retain(|&root_id| root_id != id);
        // Safety: a retained node id has a value.
        unsafe { self.runtime_state.hierarchy_nodes.release(id) };
        self.runtime_state.hierarchy_node_ids.release_stable(id);
        Ok(())
    }

    /// Removes `material_id` from `palette_id`, first repainting every live
    /// voxel that samples it onto `replacement_id` so no voxel is left
    /// without a material. Leaves a hole until [`gc`](Self::gc) renumbers.
    /// Errors, changing nothing, under the
    /// [`remove_materials`](Self::remove_materials) rules.
    pub fn remove_material(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
        replacement_id: U32Id<BVoxMaterial>,
    ) -> Result<()> {
        self.remove_materials(palette_id, &HashMap::from([(material_id, replacement_id)]))
    }

    /// Removes every keyed material of `replacement_ids` from `palette_id`,
    /// first repainting each live voxel that samples one onto the material it
    /// pairs with so no voxel is left without a material. The whole batch
    /// repaints in one pass over the voxels, so merging a palette down costs
    /// what removing a single material does. Leaves holes until
    /// [`gc`](Self::gc) renumbers. Errors, changing nothing, if:
    ///
    /// 1. `palette_id` is not one of this state's palettes
    /// 2. a material id or a replacement id is not one of that palette's
    ///    materials
    /// 3. a replacement is itself removed, which covers a material named as
    ///    its own replacement
    pub fn remove_materials(
        &mut self,
        palette_id: U32Id<BVoxPalette>,
        replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
    ) -> Result<()> {
        if !self.runtime_state.palette_ids.is_retained(palette_id) {
            return Err(Error::UnknownPalette { palette_id });
        }
        // Safety: the palette id is retained.
        let palette_ref = unsafe { self.runtime_state.palettes.get(palette_id) };
        for (&material_id, &replacement_id) in replacement_ids {
            for id in [material_id, replacement_id] {
                if !palette_ref.contains_material(id) {
                    return Err(Error::UnknownMaterial { material_id: id });
                }
            }
            if replacement_ids.contains_key(&replacement_id) {
                return Err(Error::SelfReplacement);
            }
        }

        // The doomed materials in listing order, so the removal below can walk
        // them back to front.
        let doomed_ids: Vec<_> = palette_ref
            .iter_materials()
            .filter(|material_id| replacement_ids.contains_key(material_id))
            .collect();

        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            let object = unsafe { self.runtime_state.objects.get_mut(object_id) };
            object.repaint_materials(palette_id, replacement_ids);
        }

        // Safety: the palette id is retained; each material is one of its
        // materials.
        let palette_ref = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
        // Back to front: a removal shifts the materials listed after it, so
        // dropping the last one first leaves nothing to shift and keeps the
        // batch linear where front-to-back removal is quadratic.
        for material_id in doomed_ids.into_iter().rev() {
            palette_ref.remove_material(material_id);
        }
        Ok(())
    }

    /// Removes `value_id` from `value_pool_id`, first repointing every palette
    /// cell
    /// that draws it onto `replacement_id` so no material is left without a
    /// value. Leaves a hole until [`gc`](Self::gc) renumbers. Errors,
    /// changing nothing, if:
    ///
    /// 1. `value_pool_id` is not one of this state's pools
    /// 2. `value_id` or `replacement_id` is not one of that pool's values
    /// 3. `replacement_id` is `value_id` itself
    pub fn remove_value_pool_value(
        &mut self,
        value_pool_id: U32Id<BVoxValuePool>,
        value_id: U32Id<BVoxValuePoolValue>,
        replacement_id: U32Id<BVoxValuePoolValue>,
    ) -> Result<()> {
        if !self.runtime_state.value_pool_ids.is_retained(value_pool_id) {
            return Err(Error::UnknownValuePool { value_pool_id });
        }
        // Safety: the pool id is retained.
        let value_pool = unsafe { self.runtime_state.value_pools.get(value_pool_id) };
        if !value_pool.contains_value(value_id) {
            return Err(Error::UnknownValuePoolValue { value_id });
        }
        if !value_pool.contains_value(replacement_id) {
            return Err(Error::UnknownValuePoolValue {
                value_id: replacement_id,
            });
        }
        if value_id == replacement_id {
            return Err(Error::SelfReplacement);
        }

        for palette_id in self.runtime_state.palette_ids.iter() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
            palette.repoint_value_pool_value(value_pool_id, value_id, replacement_id);
        }

        // Safety: the pool id is retained and the value is one of its values.
        unsafe { self.runtime_state.value_pools.get_mut(value_pool_id) }
            .release_value_stable(value_id);
        Ok(())
    }

    /// Compacts every id pool back to a contiguous `0..len` in listing order
    /// and rewrites every cross-reference to match, so a state edited by
    /// removals and moves numbers its entities the way a freshly loaded one
    /// does and saves stay deterministic. Call it once before saving, not after
    /// each removal or move.
    ///
    /// Requires a referentially valid state. The `remove_*` methods preserve it
    /// by detaching what they remove, and [`validate`](Self::validate) checks
    /// it. The voxel grids are dense and never compacted, so voxel ids keep
    /// equaling their raster index.
    ///
    /// Returns the [`VoxGcRemap`] recording where each id moved, so any ids
    /// held outside the state can be translated to their compacted values.
    pub fn gc(&mut self) -> VoxGcRemap {
        // Compact each pool's values first, recording the value relabelings
        // by the pool's pre-gc id so the palette pass below can translate its
        // cells before pool ids move.
        let value_pool_id_space =
            self.runtime_state.value_pool_ids.peek_next_fresh().to_u32() as usize;
        let mut value_pool_value_remaps: IdVec<BVoxValuePool, IdRemap<BVoxValuePoolValue, u32>> =
            IdVec::from_vec(
                (0..value_pool_id_space)
                    .map(|_| IdRemap::default())
                    .collect(),
            );
        for value_pool_id in self.runtime_state.value_pool_ids.iter() {
            // Safety: retained pool ids have a value.
            let value_pool = unsafe { self.runtime_state.value_pools.get_mut(value_pool_id) };
            value_pool_value_remaps[value_pool_id.to_usize_id()] = value_pool.gc_values();
        }

        // Compact the shared value-pool store, then relabel every palette
        // property's pool, so the pool ids are settled before palettes are
        // compacted. Pool ids follow the listing, so a pool moved before gc is
        // renumbered here and every property's pool id is rewritten to match.
        let value_pool_remap = self.runtime_state.value_pool_ids.gc();
        // Safety: the value-pool column was in sync with the pre-gc pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.value_pools.gc(&value_pool_remap) };

        // Compact each palette's own pools, so the material relabelings are
        // ready when object samples are translated below. They are indexed by
        // old palette id, so the column covers the palette pool's whole id
        // space. Cells translate through the value relabelings first, while
        // each property still names its pool's pre-gc id.
        let palette_id_space = self.runtime_state.palette_ids.peek_next_fresh().to_u32() as usize;
        let mut material_remaps: IdVec<BVoxPalette, IdRemap<BVoxMaterial, u32>> =
            IdVec::from_vec((0..palette_id_space).map(|_| IdRemap::default()).collect());
        for palette_id in self.runtime_state.palette_ids.iter().collect::<Vec<_>>() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get_mut(palette_id) };
            palette.relabel_value_pool_values(&value_pool_value_remaps);
            palette.relabel_value_pools(&value_pool_remap);
            material_remaps[palette_id.to_usize_id()] = palette.gc();
        }

        // Compact the palette pool.
        let palette_remap = self.runtime_state.palette_ids.gc();
        // Safety: the palette column was in sync with the pre-gc palette pool,
        // and nothing has retained or released since.
        unsafe { self.runtime_state.palettes.gc(&palette_remap) };

        // Rewrite each object's palette references and sample cells, then
        // compact its own reference pool.
        let object_ids: Vec<_> = self.runtime_state.object_ids.iter().collect();
        for object_id in object_ids {
            // Safety: retained object ids have a value.
            unsafe { self.runtime_state.objects.get_mut(object_id) }
                .gc(&palette_remap, &material_remaps);
        }

        // Compact the object pool.
        let object_remap = self.runtime_state.object_ids.gc();
        // Safety: the object column was in sync with the pre-gc object pool,
        // and nothing has retained or released since.
        unsafe { self.runtime_state.objects.gc(&object_remap) };

        // Compact the node pool, then translate child links and roots, which
        // point at the relabeled nodes and objects.
        let node_remap = self.runtime_state.hierarchy_node_ids.gc();
        // Safety: the node column was in sync with the pre-gc node pool, and
        // nothing has retained or released since.
        unsafe { self.runtime_state.hierarchy_nodes.gc(&node_remap) };

        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        for node_id in node_ids {
            // Safety: retained node ids have a value.
            let node = unsafe { self.runtime_state.hierarchy_nodes.get_mut(node_id) };
            for child_id in &mut node.child_node_ids {
                *child_id = node_remap
                    .new_id(*child_id)
                    .expect("a child node is live in a valid state");
            }
            for object_id in &mut node.child_object_ids {
                *object_id = object_remap
                    .new_id(*object_id)
                    .expect("a child object is live in a valid state");
            }
        }

        for root_id in &mut self.runtime_state.root_hierarchy_node_ids {
            *root_id = node_remap
                .new_id(*root_id)
                .expect("a root is live in a valid state");
        }

        VoxGcRemap {
            value_pools: value_pool_remap,
            value_pool_values: value_pool_value_remaps,
            objects: object_remap,
            palettes: palette_remap,
            hierarchy_nodes: node_remap,
            materials: material_remaps,
        }
    }

    /// Releases value-pool entries no material references, keeping the
    /// survivors' listing order and their ids. The value-pool-value counterpart
    /// to the entity `remove_*` methods. [`gc`](Self::gc) renumbers. Requires a
    /// referentially valid state, which [`validate`](Self::validate) checks.
    ///
    /// 1. references union across palettes, so a shared entry survives while
    ///    any one material uses it
    /// 2. a pool nothing references is left whole, since
    ///    [`validate`](Self::validate) requires every pool non-empty
    /// 3. the state stays referentially valid
    pub fn prune_value_pools(&mut self) {
        // The value ids each pool still has a material referencing.
        let value_pool_ids: Vec<_> = self.runtime_state.value_pool_ids.iter().collect();
        let mut referenced_ids: HashMap<U32Id<BVoxValuePool>, HashSet<U32Id<BVoxValuePoolValue>>> =
            value_pool_ids
                .iter()
                .map(|&id| (id, HashSet::new()))
                .collect();

        for palette_id in self.runtime_state.palette_ids.iter() {
            // Safety: retained palette ids have a value.
            let palette = unsafe { self.runtime_state.palettes.get(palette_id) };
            for (property_id, property) in palette.iter_properties() {
                let used_ids = referenced_ids
                    .get_mut(&property.value_pool_id)
                    .expect("a property names a live value pool in a valid state");
                for material_id in palette.iter_materials() {
                    let value_id = palette
                        .value_id(material_id, property_id)
                        .expect("a retained material has a value id for every property");
                    used_ids.insert(value_id);
                }
            }
        }

        // Release each pool's unreferenced entries. A pool nothing references
        // is left whole.
        for &value_pool_id in &value_pool_ids {
            let keep_ids = &referenced_ids[&value_pool_id];
            if keep_ids.is_empty() {
                continue;
            }
            // Safety: retained pool ids have a value.
            let value_pool = unsafe { self.runtime_state.value_pools.get_mut(value_pool_id) };
            let doomed_ids: Vec<_> = value_pool
                .iter_values()
                .map(|(value_id, _)| value_id)
                .filter(|value_id| !keep_ids.contains(value_id))
                .collect();
            // Back to front: a release shifts the values listed after it, so
            // dropping the last one first leaves nothing to shift and keeps
            // the prune linear where front-to-back release is quadratic.
            for value_id in doomed_ids.into_iter().rev() {
                value_pool.release_value_stable(value_id);
            }
        }
    }

    /// Reorders `value_pool_id`'s values to `new_order_ids`, which lists the
    /// pool's value ids in their new listing order. Value ids are stable, so
    /// what every material resolves to is unchanged. Errors, changing nothing,
    /// if `value_pool_id` is not one of this state's or `new_order_ids` does
    /// not list each of the pool's value ids exactly once.
    pub fn reorder_value_pool(
        &mut self,
        value_pool_id: U32Id<BVoxValuePool>,
        new_order_ids: &[U32Id<BVoxValuePoolValue>],
    ) -> Result<()> {
        if !self.runtime_state.value_pool_ids.is_retained(value_pool_id) {
            return Err(Error::UnknownValuePool { value_pool_id });
        }
        // Safety: the id is retained, so it has a value.
        unsafe { self.runtime_state.value_pools.get_mut(value_pool_id) }
            .set_value_order(new_order_ids)
            .ok_or(Error::ValuePoolValueOrder)
    }

    /// Audits the full rule set. Every rule here is also enforced at a
    /// mutation point (a constructor, an insertion, or the mutation itself),
    /// so a state reached through the public API always passes; a failure
    /// reports a voxcore bug, never a caller error. The checks stay as the
    /// specification of what the mutations preserve:
    ///
    /// 1. every value pool is non-empty, its values well-formed for its kind,
    ///    and its `min`/`max` finite, integer-valued for an `int` pool, and
    ///    ordered
    /// 2. per palette:
    ///    1. every property names a live value pool
    ///    2. no property name repeats
    ///    3. every material value id is within its property's pool
    ///    4. there is at least one material
    /// 3. every object layer references a live palette (two layers may share
    ///    one), and every live-voxel sample material is within its layer's
    ///    palette
    /// 4. every node child node and child object resolves, and no node lists
    ///    the same one twice
    /// 5. every root resolves, and no root repeats
    /// 6. every node transform has finite position and scale components, a
    ///    non-zero scale on each axis, and a unit-length rotation quaternion
    ///    within `1e-6`
    /// 7. the `child_node_ids` graph is acyclic
    ///
    /// A node may have several parents, since the hierarchy is a DAG; that
    /// sharing is not a cycle.
    pub fn validate(&self) -> Result<()> {
        // Value pools are non-empty and their values and bounds well-formed for
        // their kind. This runs first, so a palette that reads a malformed pool
        // is reported after the pool it reads.
        for (value_pool_id, value_pool) in self.iter_value_pools() {
            match value_pool.first_flaw() {
                None => {}
                Some(VoxValuePoolFlaw::Empty) => {
                    return Err(Error::EmptyValuePool { value_pool_id });
                }
                Some(VoxValuePoolFlaw::Bound) => {
                    return Err(Error::ValuePoolBound { value_pool_id });
                }
                Some(VoxValuePoolFlaw::Value(value_id)) => {
                    return Err(Error::ValuePoolValue {
                        value_pool_id,
                        value_id,
                    });
                }
            }
        }

        // Palette property rules: pools resolve, names are unique, and every
        // value id is within its pool.
        for (palette_id, palette) in self.iter_palettes() {
            let mut seen_property_names = HashSet::with_capacity(palette.property_count());
            for (property_id, property) in palette.iter_properties() {
                let value_pool =
                    self.value_pool(property.value_pool_id)
                        .ok_or(Error::PropertyValuePool {
                            palette_id,
                            property_id,
                            value_pool_id: property.value_pool_id,
                        })?;
                if !seen_property_names.insert(property.name.as_str()) {
                    return Err(Error::DuplicatePropertyName {
                        name: property.name.clone(),
                    });
                }
                for material_id in palette.iter_materials() {
                    let value_id = palette
                        .value_id(material_id, property_id)
                        .expect("a material has a value id for every property");
                    if !value_pool.contains_value(value_id) {
                        return Err(Error::MaterialValue {
                            palette_id,
                            property_id,
                            material_id,
                        });
                    }
                }
            }

            // Every palette is sampled, so it needs a material to sample.
            if palette.material_count() == 0 {
                return Err(Error::PaletteWithoutMaterials { palette_id });
            }
        }

        // Object layer palette refs and live-voxel sample materials. Checks are
        // by id retention, not index range, so they hold whether or not
        // removals have left the pools with holes. Two layers may reference the
        // same palette, so there is no duplicate-layer rule.
        for (object_id, object) in self.iter_objects() {
            let mut layer_palettes = Vec::with_capacity(object.layer_count());
            for (layer_id, palette_id) in object.iter_layers() {
                let palette = self.palette(palette_id).ok_or(Error::PaletteRef {
                    object_id,
                    palette_id,
                })?;
                layer_palettes.push((layer_id, palette));
            }
            // Every live voxel samples a material within each layer's
            // palette. Layer-major so each layer's sample column is read
            // once.
            for &(layer_id, palette) in &layer_palettes {
                let samples = object
                    .iter_live_samples(layer_id)
                    .expect("an iterated layer is one of the object's layers");
                for (voxel_id, material_id) in samples {
                    if !palette.contains_material(material_id) {
                        return Err(Error::SampleMaterial {
                            object_id,
                            voxel_id,
                            material_id,
                        });
                    }
                }
            }
        }

        // Node children; retention-checked before the cycle pass.
        for (node_id, node) in self.iter_hierarchy_nodes() {
            let mut seen_child_node_ids = HashSet::with_capacity(node.child_node_ids.len());
            for &child_id in &node.child_node_ids {
                if self.hierarchy_node(child_id).is_none() {
                    return Err(Error::ChildNode { node_id, child_id });
                }
                if !seen_child_node_ids.insert(child_id) {
                    return Err(Error::DuplicateChildNode { node_id, child_id });
                }
            }
            let mut seen_child_object_ids = HashSet::with_capacity(node.child_object_ids.len());
            for &object_id in &node.child_object_ids {
                if self.object(object_id).is_none() {
                    return Err(Error::ChildObject { node_id, object_id });
                }
                if !seen_child_object_ids.insert(object_id) {
                    return Err(Error::DuplicateChildObject { node_id, object_id });
                }
            }

            // The node transform must be finite and non-degenerate. The
            // rotation needs no finiteness guard of its own: a non-finite
            // component fails the unit-length check below.
            let position = node.transform.position;
            let scale = node.transform.scale;
            if !vector_is_finite(position) || !vector_is_finite(scale) {
                return Err(Error::NonFiniteTransform { node_id });
            }
            if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
                return Err(Error::ZeroScale { node_id });
            }
            let rotation = node.transform.rotation;
            if !rotation.is_normalized_within(UNIT_ROTATION_TOLERANCE) {
                return Err(Error::NonUnitRotation { node_id });
            }
        }

        // Roots.
        let mut seen_root_ids =
            HashSet::with_capacity(self.runtime_state.root_hierarchy_node_ids.len());
        for &root_id in &self.runtime_state.root_hierarchy_node_ids {
            if self.hierarchy_node(root_id).is_none() {
                return Err(Error::Root { root_id });
            }
            if !seen_root_ids.insert(root_id) {
                return Err(Error::DuplicateRoot { root_id });
            }
        }

        // Acyclicity; every child is now known live. Works over the retained
        // node ids by position, so it holds whether or not the pool has
        // holes.
        let node_ids: Vec<_> = self.runtime_state.hierarchy_node_ids.iter().collect();
        let index_of: HashMap<U32Id<BVoxHierarchyNode>, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(index, &id)| (id, index))
            .collect();
        let children: Vec<&[U32Id<BVoxHierarchyNode>]> = node_ids
            .iter()
            .map(|&node_id| {
                // Safety: `node_id` is a retained node id.
                unsafe { self.runtime_state.hierarchy_nodes.get(node_id) }
                    .child_node_ids
                    .as_slice()
            })
            .collect();
        if let Some(position) = first_cycle_position(&children, &index_of) {
            return Err(Error::Cycle {
                node_id: node_ids[position],
            });
        }

        Ok(())
    }

    /// Deep copy. The runtime scene rebuilds its columns against fresh id
    /// pools.
    pub fn clone_state(&self) -> Self {
        Self {
            runtime_state: self.runtime_state.clone_runtime_state(),
            ext: self.ext.clone(),
        }
    }
}

/// The `children` position of a node lying on a `child_node_ids` cycle, or `None`
/// if the graph is acyclic.
///
/// `children` holds each node's child ids at that node's position, and
/// `index_of` maps a child id back to its position. A child missing from
/// `index_of` leads outside the checked set, where no edge can return, so it is
/// skipped.
///
/// The walk is an iterative three-colour DFS, so a deep chain cannot overflow
/// the stack. A back edge into an in-progress node is a cycle; revisiting a
/// finished one is not.
fn first_cycle_position(
    children: &[&[U32Id<BVoxHierarchyNode>]],
    index_of: &HashMap<U32Id<BVoxHierarchyNode>, usize>,
) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let count = children.len();
    let mut colour = vec![WHITE; count];

    for start in 0..count {
        if colour[start] != WHITE {
            continue;
        }
        colour[start] = GREY;
        // Each frame is a node position plus how many children we have
        // walked.
        let mut stack: Vec<(usize, usize)> = vec![(start, 0)];
        while let Some(&(node, cursor)) = stack.last() {
            let node_children = children[node];
            match (cursor < node_children.len()).then(|| node_children[cursor]) {
                Some(child_id) => {
                    stack.last_mut().unwrap().1 += 1;
                    let Some(&child_position) = index_of.get(&child_id) else {
                        continue;
                    };
                    match colour[child_position] {
                        WHITE => {
                            colour[child_position] = GREY;
                            stack.push((child_position, 0));
                        }
                        GREY => return Some(child_position),
                        _ => {}
                    }
                }
                None => {
                    colour[node] = BLACK;
                    stack.pop();
                }
            }
        }
    }

    None
}

/// Whether every component of `vector` is finite.
fn vector_is_finite(vector: TyVector3F64) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use crate::{
        BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty,
        BVoxValuePool, BVoxValuePoolValue, BVoxVoxel, Error, Result, VoxBound, VoxHierarchyNode,
        VoxMain, VoxObject, VoxPalette, VoxValuePool, VoxValuePoolKind, VoxValuePoolValueRef,
    };
    use branded_id::U32Id;
    use std::collections::HashMap;
    use ty_math::{TyQuaternion, TyVector3, TyVector3I32, TyVector3U32};

    fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn material_id(index: u32) -> U32Id<BVoxMaterial> {
        U32Id::from_u32(index)
    }

    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    fn value_pool_id(index: u32) -> U32Id<BVoxValuePool> {
        U32Id::from_u32(index)
    }

    fn palette_id(index: u32) -> U32Id<BVoxPalette> {
        U32Id::from_u32(index)
    }

    fn voxel_id(index: u32) -> U32Id<BVoxVoxel> {
        U32Id::from_u32(index)
    }

    /// A node referencing the given child nodes (and no objects).
    fn node_with_children(child_node_ids: Vec<U32Id<BVoxHierarchyNode>>) -> VoxHierarchyNode {
        VoxHierarchyNode {
            child_node_ids,
            ..VoxHierarchyNode::default()
        }
    }

    /// A node placing the given child objects (and no child nodes).
    fn node_with_objects(child_object_ids: Vec<U32Id<BVoxObject>>) -> VoxHierarchyNode {
        VoxHierarchyNode {
            child_object_ids,
            ..VoxHierarchyNode::default()
        }
    }

    /// A 1x1x1 object with its single voxel live, so its grid is exactly tight.
    fn unit_object(name: &str) -> VoxObject {
        let mut object = VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(voxel_id, &[]).unwrap();
        object
    }

    /// Adds an unbounded `int` value pool holding `values` and returns its id.
    fn int_value_pool(state: &mut VoxMain, values: Vec<i64>) -> U32Id<BVoxValuePool> {
        state.add_value_pool(VoxValuePool::int(VoxBound::None, VoxBound::None, values).unwrap())
    }

    /// A palette holding one empty material and no properties, the smallest
    /// shape [`VoxMain::add_palette`] accepts.
    fn bare_palette() -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        palette
    }

    /// A palette with one property "v" on `value_pool_id` and two materials,
    /// drawing value ids 0 and 1.
    fn two_material_palette(value_pool_id: U32Id<BVoxValuePool>) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette
            .add_property("v".to_owned(), value_pool_id, value_id(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        palette.add_material(vec![value_id(1)]).unwrap();
        palette
    }

    /// A palette with one property "v" on `value_pool_id` and one material
    /// drawing value id `index`.
    fn one_material_palette(value_pool_id: U32Id<BVoxValuePool>, index: u32) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette
            .add_property("v".to_owned(), value_pool_id, value_id(0))
            .unwrap();
        palette.add_material(vec![value_id(index)]).unwrap();
        palette
    }

    #[test]
    fn add_and_read_back_in_listing_order() {
        let mut state = VoxMain::default();
        let a_id = state.add_object(unit_object("a")).unwrap();
        let b_id = state.add_object(unit_object("b")).unwrap();

        assert_eq!(state.object_count(), 2);
        assert_eq!(state.object(a_id).unwrap().name(), "a");
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["a", "b"]);
        assert_eq!(b_id.to_u32(), 1);
    }

    #[test]
    fn add_and_read_back_value_pools_in_listing_order() {
        let mut state = VoxMain::default();
        let colors_id =
            state.add_value_pool(VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0]]).unwrap());
        let metallic_id = state.add_value_pool(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.0, 1.0])
                .unwrap(),
        );

        assert_eq!(state.value_pool_count(), 2);
        assert_eq!(colors_id, U32Id::<BVoxValuePool>::from_u32(0));
        assert_eq!(metallic_id.to_u32(), 1);
        assert!(matches!(
            state.value_pool(colors_id).map(VoxValuePool::kind),
            Some(VoxValuePoolKind::Srgba { .. })
        ));
        assert_eq!(
            state.value_pool(metallic_id).map(VoxValuePool::values_len),
            Some(2)
        );
        // An id past the pool is not one of this state's.
        assert_eq!(state.value_pool(U32Id::<BVoxValuePool>::from_u32(2)), None);

        let mut value_pools = state.iter_value_pools();
        assert!(matches!(
            value_pools.next().map(|(_, value_pool)| value_pool.kind()),
            Some(VoxValuePoolKind::Srgba { .. })
        ));
        assert!(matches!(
            value_pools.next().map(|(_, value_pool)| value_pool.kind()),
            Some(VoxValuePoolKind::Float { .. })
        ));
        assert!(value_pools.next().is_none());
    }

    #[test]
    fn clone_state_deep_copies_value_pools() {
        let mut state = VoxMain::default();
        state.add_value_pool(VoxValuePool::int(VoxBound::None, VoxBound::None, vec![7]).unwrap());

        let copy = state.clone_state();
        assert_eq!(copy.value_pool_count(), 1);
        assert_eq!(
            copy.value_pool(U32Id::<BVoxValuePool>::from_u32(0)),
            Some(&VoxValuePool::int(VoxBound::None, VoxBound::None, vec![7]).unwrap())
        );

        // Mutating the original must not touch the copy.
        state.add_value_pool(VoxValuePool::boolean(vec![true]).unwrap());
        assert_eq!(state.value_pool_count(), 2);
        assert_eq!(copy.value_pool_count(), 1);
    }

    #[test]
    fn prune_value_pools_releases_unreferenced_entries_keeping_ids() {
        let mut state = VoxMain::default();
        // Four colors; the palette references only the middle two.
        let colors_id = state.add_value_pool(
            VoxValuePool::srgba(vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red, unused
                [0.0, 1.0, 0.0, 1.0], // 1 green, used
                [0.0, 0.0, 1.0, 1.0], // 2 blue, unused
                [1.0, 1.0, 1.0, 1.0], // 3 white, used
            ])
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        let property_id = palette
            .add_property("baseColorFactor".to_owned(), colors_id, value_id(0))
            .unwrap();
        let green_id = palette.add_material(vec![value_id(1)]).unwrap();
        let white_id = palette.add_material(vec![value_id(3)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        state.validate().unwrap();

        state.prune_value_pools();

        // The pool keeps green then white in listing order, and the material
        // cells keep their ids. gc owns the renumbering.
        assert_eq!(
            state.value_pool(colors_id),
            Some(&VoxValuePool::srgba(vec![[0.0, 1.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]]).unwrap())
        );
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(palette.value_id(green_id, property_id), Some(value_id(1)));
        assert_eq!(palette.value_id(white_id, property_id), Some(value_id(3)));
        state.validate().unwrap();

        // gc renumbers the survivors to listing order: green to 0, white to 1.
        state.gc();
        let palette = state.palette(palette_id).unwrap();
        assert_eq!(palette.value_id(green_id, property_id), Some(value_id(0)));
        assert_eq!(palette.value_id(white_id, property_id), Some(value_id(1)));
        state.validate().unwrap();
    }

    #[test]
    fn prune_value_pools_keeps_entries_any_palette_still_uses() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20, 30]);
        // Palette a draws id 0, palette b draws id 2, and id 1 is unused.
        let mut a = VoxPalette::default();
        let a_property_id = a
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let a_material_id = a.add_material(vec![value_id(0)]).unwrap();
        let a_id = state.add_palette(a).unwrap();
        let mut b = VoxPalette::default();
        let b_property_id = b
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let b_material_id = b.add_material(vec![value_id(2)]).unwrap();
        let b_id = state.add_palette(b).unwrap();
        state.validate().unwrap();

        state.prune_value_pools();

        // 10 and 30 survive (ids 0 and 2 used). 20 (id 1) is dropped, and the
        // survivors keep their ids until gc.
        assert_eq!(
            state.value_pool(ints_id),
            Some(&VoxValuePool::int(VoxBound::None, VoxBound::None, vec![10, 30]).unwrap())
        );
        assert_eq!(
            state
                .palette(a_id)
                .unwrap()
                .value_id(a_material_id, a_property_id),
            Some(value_id(0))
        );
        assert_eq!(
            state
                .palette(b_id)
                .unwrap()
                .value_id(b_material_id, b_property_id),
            Some(value_id(2))
        );
        state.validate().unwrap();
    }

    #[test]
    fn reorder_value_pool_permutes_the_listing_leaving_resolutions() {
        let mut state = VoxMain::default();
        // Three colors. Two palettes bind the pool, each with materials
        // drawing scattered ids.
        let colors_id = state.add_value_pool(
            VoxValuePool::srgba(vec![
                [1.0, 0.0, 0.0, 1.0], // 0 red
                [0.0, 1.0, 0.0, 1.0], // 1 green
                [0.0, 0.0, 1.0, 1.0], // 2 blue
            ])
            .unwrap(),
        );
        let mut a = VoxPalette::default();
        let a_property_id = a
            .add_property("baseColorFactor".to_owned(), colors_id, value_id(0))
            .unwrap();
        let a_blue_id = a.add_material(vec![value_id(2)]).unwrap();
        let a_red_id = a.add_material(vec![value_id(0)]).unwrap();
        let a_id = state.add_palette(a).unwrap();
        let mut b = VoxPalette::default();
        let b_property_id = b
            .add_property("baseColorFactor".to_owned(), colors_id, value_id(0))
            .unwrap();
        let b_green_id = b.add_material(vec![value_id(1)]).unwrap();
        let b_id = state.add_palette(b).unwrap();
        state.validate().unwrap();

        // List blue first, then red, then green.
        assert_eq!(
            state.reorder_value_pool(colors_id, &[value_id(2), value_id(0), value_id(1)]),
            Ok(())
        );

        // The pool follows the new order.
        assert_eq!(
            state.value_pool(colors_id),
            Some(
                &VoxValuePool::srgba(vec![
                    [0.0, 0.0, 1.0, 1.0],
                    [1.0, 0.0, 0.0, 1.0],
                    [0.0, 1.0, 0.0, 1.0]
                ])
                .unwrap()
            )
        );
        // No cell is rewritten: value ids are stable, so every material keeps
        // its id and resolves to its original color.
        let a = state.palette(a_id).unwrap();
        assert_eq!(a.value_id(a_blue_id, a_property_id), Some(value_id(2)));
        assert_eq!(a.value_id(a_red_id, a_property_id), Some(value_id(0)));
        let value_pool = state.value_pool(colors_id).unwrap();
        assert_eq!(
            value_pool.value(value_id(2)),
            Some(VoxValuePoolValueRef::Srgba(&[0.0, 0.0, 1.0, 1.0]))
        );
        assert_eq!(
            state
                .palette(b_id)
                .unwrap()
                .value_id(b_green_id, b_property_id),
            Some(value_id(1))
        );
        state.validate().unwrap();
    }

    #[test]
    fn reorder_value_pool_rejects_a_non_permutation_without_changing_state() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20, 30]);

        // A repeated id, a wrong length, an id not the pool's, and an unknown
        // pool all reject.
        assert_eq!(
            state.reorder_value_pool(ints_id, &[value_id(0), value_id(0), value_id(1)]),
            Err(Error::ValuePoolValueOrder)
        );
        assert_eq!(
            state.reorder_value_pool(ints_id, &[value_id(0), value_id(1)]),
            Err(Error::ValuePoolValueOrder)
        );
        assert_eq!(
            state.reorder_value_pool(ints_id, &[value_id(0), value_id(1), value_id(3)]),
            Err(Error::ValuePoolValueOrder)
        );
        assert_eq!(
            state.reorder_value_pool(
                U32Id::<BVoxValuePool>::from_u32(9),
                &[value_id(0), value_id(1), value_id(2)]
            ),
            Err(Error::UnknownValuePool {
                value_pool_id: value_pool_id(9)
            })
        );
        assert_eq!(
            state.value_pool(ints_id),
            Some(&VoxValuePool::int(VoxBound::None, VoxBound::None, vec![10, 20, 30]).unwrap())
        );
    }

    #[test]
    fn prune_value_pools_leaves_a_fully_referenced_value_pool() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![1, 2]);
        let mut palette = VoxPalette::default();
        palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        palette.add_material(vec![value_id(1)]).unwrap();
        state.add_palette(palette).unwrap();

        state.prune_value_pools();

        assert_eq!(
            state.value_pool(ints_id).map(VoxValuePool::values_len),
            Some(2)
        );
    }

    #[test]
    fn validate_accepts_a_shared_child_dag() {
        let mut state = VoxMain::default();
        let leaf_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        // Sharing a child across parents is legal in a DAG; each parent lists
        // it once.
        let a_id = state
            .add_hierarchy_node(node_with_children(vec![leaf_id]))
            .unwrap();
        let b_id = state
            .add_hierarchy_node(node_with_children(vec![leaf_id]))
            .unwrap();
        state.set_root_hierarchy_node_ids(vec![a_id, b_id]).unwrap();

        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn add_hierarchy_node_rejects_a_duplicate_child_node() {
        let mut state = VoxMain::default();
        let leaf_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        assert_eq!(
            state.add_hierarchy_node(node_with_children(vec![leaf_id, leaf_id])),
            Err(Error::InsertedDuplicateChildNode {
                index: 0,
                child_id: leaf_id,
            })
        );
        assert_eq!(state.hierarchy_node_count(), 1);
    }

    #[test]
    fn add_hierarchy_node_rejects_a_duplicate_child_object() {
        let mut state = VoxMain::default();
        let object_id = state.add_object(unit_object("o")).unwrap();
        assert_eq!(
            state.add_hierarchy_node(node_with_objects(vec![object_id, object_id])),
            Err(Error::InsertedDuplicateChildObject {
                index: 0,
                object_id,
            })
        );
        assert_eq!(state.hierarchy_node_count(), 0);
    }

    #[test]
    fn root_setters_reject_a_duplicate_root() {
        let mut state = VoxMain::default();
        let node_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        assert_eq!(
            state.set_root_hierarchy_node_ids(vec![node_id, node_id]),
            Err(Error::DuplicateRoot { root_id: node_id })
        );
        assert_eq!(state.root_hierarchy_node_ids(), []);

        state.push_root_hierarchy_node_id(node_id).unwrap();
        assert_eq!(
            state.push_root_hierarchy_node_id(node_id),
            Err(Error::DuplicateRoot { root_id: node_id })
        );
        assert_eq!(state.root_hierarchy_node_ids(), [node_id]);
    }

    #[test]
    fn validate_accepts_a_palette_with_no_properties() {
        let mut state = VoxMain::default();
        // A palette with no properties still carries materials; each
        // row is empty and every property resolves to its default. Voxels
        // sample them like any other material.
        let mut palette = VoxPalette::default();
        palette.add_material(vec![]).unwrap();
        let second_id = palette.add_material(vec![]).unwrap();
        let live_palette_id = state.add_palette(palette).unwrap();

        let mut object = unit_object("o");
        object.add_layer(live_palette_id, second_id);
        state.add_object(object).unwrap();
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn add_palette_rejects_an_empty_palette() {
        let mut state = VoxMain::default();
        // Every palette is sampled, so even a property-less palette needs a
        // material.
        assert_eq!(
            state.add_palette(VoxPalette::default()),
            Err(Error::NoPaletteMaterials)
        );
        assert_eq!(state.palette_count(), 0);
    }

    #[test]
    fn add_object_rejects_a_dangling_layer_palette() {
        let mut state = VoxMain::default();
        let mut object = unit_object("o");
        // Reference palette id 0, but the state has no palettes.
        let layer_id = object.add_layer(palette_id(0), material_id(0));

        assert_eq!(
            state.add_object(object),
            Err(Error::LayerPaletteRef {
                layer_id,
                palette_id: palette_id(0),
            })
        );
        assert_eq!(state.object_count(), 0);
    }

    #[test]
    fn add_object_rejects_a_bad_sample_material() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![7]);
        let live_palette_id = state.add_palette(one_material_palette(ints_id, 0)).unwrap();

        // The layer back-fills the live voxel with material 9, beyond the
        // palette's one material.
        let mut object = unit_object("o");
        let layer_id = object.add_layer(live_palette_id, material_id(9));

        assert_eq!(
            state.add_object(object),
            Err(Error::LayerSampleMaterial {
                layer_id,
                voxel_id: voxel_id(0),
                material_id: material_id(9),
            })
        );
        assert_eq!(state.object_count(), 0);
    }

    #[test]
    fn add_hierarchy_node_rejects_a_dangling_child() {
        let mut state = VoxMain::default();
        assert_eq!(
            state.add_hierarchy_node(node_with_children(vec![node_id(9)])),
            Err(Error::UnknownHierarchyNode {
                node_id: node_id(9)
            })
        );
        assert_eq!(state.hierarchy_node_count(), 0);
    }

    #[test]
    fn root_setters_reject_a_dangling_root() {
        let mut state = VoxMain::default();
        state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        assert_eq!(
            state.set_root_hierarchy_node_ids(vec![node_id(7)]),
            Err(Error::Root {
                root_id: node_id(7)
            })
        );
        assert_eq!(
            state.push_root_hierarchy_node_id(node_id(7)),
            Err(Error::Root {
                root_id: node_id(7)
            })
        );
        assert_eq!(state.root_hierarchy_node_ids(), []);
    }

    #[test]
    fn add_hierarchy_nodes_rejects_a_cycle() {
        let mut state = VoxMain::default();
        // node 0 -> child 1, node 1 -> child 0.
        assert!(matches!(
            state.add_hierarchy_nodes(vec![
                node_with_children(vec![node_id(1)]),
                node_with_children(vec![node_id(0)]),
            ]),
            Err(Error::InsertedCycle { .. })
        ));
        assert_eq!(state.hierarchy_node_count(), 0);
    }

    #[test]
    fn add_hierarchy_nodes_accepts_forward_references() {
        let mut state = VoxMain::default();
        // node 0 lists node 1 before node 1 exists; the batch resolves it.
        let ids = state
            .add_hierarchy_nodes(vec![
                node_with_children(vec![node_id(1)]),
                VoxHierarchyNode::default(),
            ])
            .unwrap();
        assert_eq!(ids, [node_id(0), node_id(1)]);
        state.set_root_hierarchy_node_ids(vec![ids[0]]).unwrap();
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn add_hierarchy_node_rejects_a_zero_scale() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        node.transform.scale = TyVector3::new(1.0, 0.0, 1.0);
        assert_eq!(
            state.add_hierarchy_node(node),
            Err(Error::InsertedZeroScale { index: 0 })
        );
    }

    #[test]
    fn add_hierarchy_node_rejects_a_non_finite_scale() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        // NaN slips past the zero-scale check (NaN == 0.0 is false), so the
        // finiteness check must catch it first.
        node.transform.scale = TyVector3::new(1.0, f64::NAN, 1.0);
        assert_eq!(
            state.add_hierarchy_node(node),
            Err(Error::InsertedNonFiniteTransform { index: 0 })
        );
    }

    #[test]
    fn add_hierarchy_node_rejects_a_non_finite_position() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        node.transform.position = TyVector3::new(0.0, 0.0, f64::INFINITY);
        assert_eq!(
            state.add_hierarchy_node(node),
            Err(Error::InsertedNonFiniteTransform { index: 0 })
        );
    }

    #[test]
    fn add_hierarchy_node_rejects_a_non_unit_rotation() {
        let mut state = VoxMain::default();
        let mut node = VoxHierarchyNode::default();
        // Length squared 4, well outside the unit tolerance.
        node.transform.rotation = TyQuaternion::from_xyzw(0.0, 0.0, 0.0, 2.0);
        assert_eq!(
            state.add_hierarchy_node(node),
            Err(Error::InsertedNonUnitRotation { index: 0 })
        );
    }

    #[test]
    fn clone_state_is_an_independent_deep_copy() {
        let mut state = VoxMain::default();
        state.add_palette(bare_palette()).unwrap();
        state.add_object(unit_object("o")).unwrap();

        let copy = state.clone_state();
        assert_eq!(copy.palette_count(), 1);
        assert_eq!(copy.object_count(), 1);

        state.add_object(unit_object("p")).unwrap();
        assert_eq!(state.object_count(), 2);
        assert_eq!(copy.object_count(), 1);
    }

    #[test]
    fn remove_object_and_palette_then_gc_renumbers_and_resolves() {
        let mut state = VoxMain::default();
        let value_pool_a_id = int_value_pool(&mut state, vec![10]);
        let value_pool_b_id = int_value_pool(&mut state, vec![20]);
        let palette_a_id = state
            .add_palette(one_material_palette(value_pool_a_id, 0))
            .unwrap();
        let palette_b_id = state
            .add_palette(one_material_palette(value_pool_b_id, 0))
            .unwrap();

        let mut a = unit_object("a");
        a.add_layer(palette_a_id, material_id(0));
        let object_a_id = state.add_object(a).unwrap();

        let mut b = unit_object("b");
        b.add_layer(palette_b_id, material_id(0));
        let live_voxel_id = b.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        b.retain_voxel(live_voxel_id, &[material_id(0)]).unwrap();
        let object_b_id = state.add_object(b).unwrap();

        let inner_id = state
            .add_hierarchy_node(node_with_objects(vec![object_a_id, object_b_id]))
            .unwrap();
        let outer_id = state
            .add_hierarchy_node(node_with_children(vec![inner_id]))
            .unwrap();
        state.set_root_hierarchy_node_ids(vec![outer_id]).unwrap();
        assert_eq!(state.validate(), Ok(()));

        // Remove object `a` and palette A; the state stays clean (no dangling
        // refs) even before gc, just with holes.
        assert_eq!(state.remove_object(object_a_id), Ok(()));
        assert_eq!(state.remove_palette(palette_a_id), Ok(()));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.object_count(), 1);
        assert_eq!(state.palette_count(), 1);

        let remap = state.gc();
        assert_eq!(state.validate(), Ok(()));

        // The survivors renumber to 0 and their cross-references follow.
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        let live_palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let property_id = U32Id::<BVoxProperty>::from_u32(0);
        assert_eq!(state.object(object_id).unwrap().name(), "b");
        // Material 0 resolves through property 0 to pool B's value 20.
        let (value_pool, resolved_value_id) = state
            .material_value(live_palette_id, material_id(0), property_id)
            .unwrap();
        assert_eq!(
            value_pool.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Int(20))
        );
        assert_eq!(
            state
                .object(object_id)
                .unwrap()
                .iter_layers()
                .collect::<Vec<_>>(),
            [(U32Id::<BVoxLayer>::from_u32(0), live_palette_id)]
        );
        assert_eq!(
            state
                .object(object_id)
                .unwrap()
                .voxel_material(live_voxel_id, U32Id::<BVoxLayer>::from_u32(0)),
            Some(material_id(0))
        );

        // The inner node dropped `a` and renumbered `b` to 0; the roots are
        // intact.
        let inner_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        assert_eq!(
            state.hierarchy_node(inner_id).unwrap().child_object_ids,
            [U32Id::<BVoxObject>::from_u32(0)]
        );
        assert_eq!(
            state.root_hierarchy_node_ids(),
            [U32Id::<BVoxHierarchyNode>::from_u32(1)]
        );

        // The returned remap translates the same renumbering for held ids:
        // removed entities map to None, survivors to their compacted ids. Value
        // pools are never removed, so both map to themselves.
        assert_eq!(remap.objects.new_id(object_a_id), None);
        assert_eq!(remap.objects.new_id(object_b_id), Some(object_id));
        assert_eq!(remap.palettes.new_id(palette_a_id), None);
        assert_eq!(remap.palettes.new_id(palette_b_id), Some(live_palette_id));
        assert!(remap.materials[palette_a_id.to_usize_id()].is_empty());
        assert_eq!(
            remap.materials[palette_b_id.to_usize_id()].new_id(material_id(0)),
            Some(material_id(0))
        );
        assert_eq!(
            remap.value_pools.new_id(value_pool_a_id),
            Some(value_pool_a_id)
        );
        assert_eq!(
            remap.value_pools.new_id(value_pool_b_id),
            Some(value_pool_b_id)
        );
    }

    #[test]
    fn remove_hierarchy_node_detaches_children_and_roots() {
        let mut state = VoxMain::default();
        let leaf_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        let mid_id = state
            .add_hierarchy_node(node_with_children(vec![leaf_id]))
            .unwrap();
        let top_id = state
            .add_hierarchy_node(node_with_children(vec![mid_id, leaf_id]))
            .unwrap();
        state
            .set_root_hierarchy_node_ids(vec![top_id, mid_id])
            .unwrap();

        assert_eq!(state.remove_hierarchy_node(mid_id), Ok(()));
        assert_eq!(
            state.remove_hierarchy_node(mid_id),
            Err(Error::UnknownHierarchyNode { node_id: mid_id })
        ); // already gone

        // `mid_id` is detached from `top_id` and the roots; the shared
        // `leaf_id` survives.
        assert_eq!(
            state.hierarchy_node(top_id).unwrap().child_node_ids,
            [leaf_id]
        );
        assert_eq!(state.root_hierarchy_node_ids(), [top_id]);
        assert!(state.hierarchy_node(mid_id).is_none());
        assert!(state.hierarchy_node(leaf_id).is_some());
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn remove_material_repaints_live_voxels_onto_the_replacement() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![0, 1]);
        let mut palette = VoxPalette::default();
        palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let keep_id = palette.add_material(vec![value_id(0)]).unwrap();
        let drop_id = palette.add_material(vec![value_id(1)]).unwrap();
        let live_palette_id = state.add_palette(palette).unwrap();

        let mut object = unit_object("o");
        let layer_id = object.add_layer(live_palette_id, keep_id);
        let live_voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object.retain_voxel(live_voxel_id, &[drop_id]).unwrap();
        let object_id = state.add_object(object).unwrap();

        // Removing `drop_id` repaints the voxel that used it onto `keep_id`.
        assert_eq!(
            state.remove_material(live_palette_id, drop_id, keep_id),
            Ok(())
        );
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(
            state
                .object(object_id)
                .unwrap()
                .voxel_material(live_voxel_id, layer_id),
            Some(keep_id)
        );
        assert!(
            !state
                .palette(live_palette_id)
                .unwrap()
                .contains_material(drop_id)
        );

        // A no-op replacement and unknown ids are rejected.
        assert_eq!(
            state.remove_material(live_palette_id, keep_id, keep_id),
            Err(Error::SelfReplacement)
        );
        assert_eq!(
            state.remove_material(live_palette_id, drop_id, keep_id),
            Err(Error::UnknownMaterial {
                material_id: drop_id
            })
        ); // drop_id gone

        state.gc();
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn remove_materials_merges_a_batch_in_one_pass() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![0, 1, 2, 3]);
        let mut palette = VoxPalette::default();
        palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let material_ids: Vec<_> = (0..4)
            .map(|index| palette.add_material(vec![value_id(index)]).unwrap())
            .collect();
        let live_palette_id = state.add_palette(palette).unwrap();

        // A four-voxel row, one voxel per material.
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(4, 1, 1)).unwrap();
        let layer_id = object.add_layer(live_palette_id, material_ids[0]);
        let voxel_ids: Vec<_> = (0..4)
            .map(|x| object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap())
            .collect();
        for (&live_voxel_id, &material_id) in voxel_ids.iter().zip(&material_ids) {
            object.retain_voxel(live_voxel_id, &[material_id]).unwrap();
        }
        let object_id = state.add_object(object).unwrap();

        // A replacement that is itself removed is rejected whole: the batch
        // would leave the chained voxels pointing at a dropped material.
        let chained_ids = HashMap::from([
            (material_ids[1], material_ids[0]),
            (material_ids[0], material_ids[3]),
            (material_ids[2], material_ids[3]),
        ]);
        assert_eq!(
            state.remove_materials(live_palette_id, &chained_ids),
            Err(Error::SelfReplacement)
        );
        assert_eq!(state.palette(live_palette_id).unwrap().material_count(), 4);

        // The batch drops materials 0 through 2 onto 3 in one pass.
        let merged_ids = HashMap::from([
            (material_ids[0], material_ids[3]),
            (material_ids[1], material_ids[3]),
            (material_ids[2], material_ids[3]),
        ]);
        assert_eq!(state.remove_materials(live_palette_id, &merged_ids), Ok(()));
        assert_eq!(state.validate(), Ok(()));
        assert_eq!(state.palette(live_palette_id).unwrap().material_count(), 1);
        for &live_voxel_id in &voxel_ids {
            assert_eq!(
                state
                    .object(object_id)
                    .unwrap()
                    .voxel_material(live_voxel_id, layer_id),
                Some(material_ids[3])
            );
        }

        state.gc();
        assert_eq!(state.validate(), Ok(()));
    }

    #[test]
    fn validate_and_gc_handle_a_high_id_sample_after_a_material_hole() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![0, 1, 2]);
        let mut palette = VoxPalette::default();
        palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let first_id = palette.add_material(vec![value_id(0)]).unwrap();
        let second_id = palette.add_material(vec![value_id(1)]).unwrap();
        let third_id = palette.add_material(vec![value_id(2)]).unwrap();
        let live_palette_id = state.add_palette(palette).unwrap();

        let mut object = unit_object("o");
        let layer_id = object.add_layer(live_palette_id, first_id);
        let live_voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        // The voxel samples the highest id.
        object.retain_voxel(live_voxel_id, &[third_id]).unwrap();
        let object_id = state.add_object(object).unwrap();

        // Remove `first_id`; no live voxel used it, so the repaint is a no-op.
        // The palette is now holed: the voxel still samples `third_id`, whose
        // id exceeds the live material count. A range check would wrongly
        // reject this; the retention check accepts it.
        assert_eq!(
            state.remove_material(live_palette_id, first_id, second_id),
            Ok(())
        );
        assert_eq!(state.validate(), Ok(()));

        state.gc();
        assert_eq!(state.validate(), Ok(()));
        // gc preserves which material the voxel samples: still the value-2
        // material, just renumbered.
        let sampled_id = state
            .object(object_id)
            .unwrap()
            .voxel_material(live_voxel_id, layer_id)
            .unwrap();
        let property_id = U32Id::<BVoxProperty>::from_u32(0);
        let (value_pool, resolved_value_id) = state
            .material_value(live_palette_id, sampled_id, property_id)
            .unwrap();
        assert_eq!(
            value_pool.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Int(2))
        );
        assert_eq!(state.palette(live_palette_id).unwrap().material_count(), 2);
    }

    #[test]
    fn remove_object_rejects_an_unknown_id() {
        let mut state = VoxMain::default();
        let object_id = state.add_object(unit_object("o")).unwrap();
        assert_eq!(state.remove_object(object_id), Ok(()));
        assert_eq!(
            state.remove_object(object_id),
            Err(Error::UnknownObject { object_id })
        );
        assert_eq!(
            state.remove_palette(U32Id::<BVoxPalette>::from_u32(0)),
            Err(Error::UnknownPalette {
                palette_id: palette_id(0)
            })
        );
    }

    #[test]
    fn objects_with_build_volume_margin_validate_and_survive_gc() {
        let mut state = VoxMain::default();
        let a_id = state
            .add_object(VoxObject::new("a".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap())
            .unwrap();
        // `b` carries margin: a 5x5x5 build volume with one live voxel off the
        // origin, which the bounds rule allows.
        let mut object_b = VoxObject::new("b".to_owned(), TyVector3U32::new(5, 5, 5)).unwrap();
        let live_voxel_id = object_b.voxel_id(TyVector3U32::new(2, 3, 1)).unwrap();
        object_b.retain_voxel(live_voxel_id, &[]).unwrap();
        let b_id = state.add_object(object_b).unwrap();
        assert_eq!(b_id.to_u32(), 1);
        assert_eq!(state.validate(), Ok(()));

        // Remove `a` and gc: `b` renumbers to 0, keeping its margin grid and
        // voxel.
        assert_eq!(state.remove_object(a_id), Ok(()));
        state.gc();
        let b0_id = U32Id::<BVoxObject>::from_u32(0);
        let object = state.object(b0_id).unwrap();
        assert_eq!(object.name(), "b");
        assert_eq!(object.bounds(), TyVector3U32::new(5, 5, 5));
        assert_eq!(
            object.live_extent(),
            Some((TyVector3U32::new(2, 3, 1), TyVector3U32::new(1, 1, 1)))
        );
    }

    /// Two layers referencing the same palette is legal and the layers do not
    /// merge, so `validate` accepts the shape and each layer resolves its own
    /// samples through it.
    #[test]
    fn two_layer_object_sharing_a_palette_gcs_and_resolves() {
        // The Phase 3 gate: build a two-layer object that shares one palette,
        // validate it, gc it, and read a material's resolved values back.
        let mut state = VoxMain::default();
        let colors_id = state.add_value_pool(
            VoxValuePool::srgba(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0]]).unwrap(),
        );
        let metallic_id = state.add_value_pool(
            VoxValuePool::float(VoxBound::Number(0.0), VoxBound::Number(1.0), vec![0.0, 1.0])
                .unwrap(),
        );
        let mut palette = VoxPalette::default();
        let color_id = palette
            .add_property("baseColorFactor".to_owned(), colors_id, value_id(0))
            .unwrap();
        let metal_id = palette
            .add_property("metallicFactor".to_owned(), metallic_id, value_id(0))
            .unwrap();
        let matte_red_id = palette
            .add_material(vec![value_id(0), value_id(0)])
            .unwrap();
        let shiny_green_id = palette
            .add_material(vec![value_id(1), value_id(1)])
            .unwrap();
        let live_palette_id = state.add_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        // Two layers on the same palette; each voxel samples one material per
        // layer.
        let base_id = object.add_layer(live_palette_id, matte_red_id);
        let overlay_id = object.add_layer(live_palette_id, matte_red_id);
        let v0_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let v1_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object
            .retain_voxel(v0_id, &[matte_red_id, shiny_green_id])
            .unwrap();
        object
            .retain_voxel(v1_id, &[shiny_green_id, matte_red_id])
            .unwrap();
        state.add_object(object).unwrap();

        assert_eq!(state.validate(), Ok(()));
        state.gc();
        assert_eq!(state.validate(), Ok(()));

        // Resolve the base-layer material at v0: matte_red draws color id 0
        // (red) and metallic id 0 (0.0).
        let object = state.object(U32Id::<BVoxObject>::from_u32(0)).unwrap();
        let sampled_id = object.voxel_material(v0_id, base_id).unwrap();
        let (value_pool, resolved_value_id) = state
            .material_value(live_palette_id, sampled_id, color_id)
            .unwrap();
        assert_eq!(
            value_pool.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Srgba(&[1.0, 0.0, 0.0, 1.0]))
        );
        let (value_pool, resolved_value_id) = state
            .material_value(live_palette_id, sampled_id, metal_id)
            .unwrap();
        assert_eq!(
            value_pool.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Float(0.0))
        );

        // The overlay layer at v0 samples shiny_green, drawing color id 1
        // (green), proving the two layers resolve independently.
        let overlay_sampled_id = object.voxel_material(v0_id, overlay_id).unwrap();
        let (value_pool, resolved_value_id) = state
            .material_value(live_palette_id, overlay_sampled_id, color_id)
            .unwrap();
        assert_eq!(
            value_pool.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Srgba(&[0.0, 1.0, 0.0, 1.0]))
        );
    }

    #[test]
    fn add_palette_rejects_a_dangling_property_value_pool() {
        let mut state = VoxMain::default();
        let mut palette = VoxPalette::default();
        // The property references pool id 0, but the state holds no pools.
        let property_id = palette
            .add_property("baseColorFactor".to_owned(), value_pool_id(0), value_id(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        assert_eq!(
            state.add_palette(palette),
            Err(Error::PropertyValuePoolRef {
                property_id,
                value_pool_id: value_pool_id(0),
            })
        );
        assert_eq!(state.palette_count(), 0);
    }

    #[test]
    fn add_palette_rejects_a_material_value_id_not_in_the_value_pool() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![0, 1]);
        let mut palette = VoxPalette::default();
        let property_id = palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        // The pool holds two values, but this material draws value id 2.
        let material_id = palette.add_material(vec![value_id(2)]).unwrap();
        assert_eq!(
            state.add_palette(palette),
            Err(Error::MaterialValueRef {
                property_id,
                material_id,
            })
        );
        assert_eq!(state.palette_count(), 0);
    }

    #[test]
    fn validate_reports_a_stale_value_id_after_a_missed_rewrite() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20]);
        let mut palette = VoxPalette::default();
        let property_id = palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let live_material_id = palette.add_material(vec![value_id(1)]).unwrap();
        let live_palette_id = state.add_palette(palette).unwrap();
        state.validate().unwrap();

        // Release the drawn value directly, skipping the cell rewrite
        // remove_value_pool_value performs, so the material's cell holds a stale id.
        // Safety: the pool id is retained.
        let value_pool_ref = unsafe { state.runtime_state.value_pools.get_mut(ints_id) };
        value_pool_ref.release_value_stable(value_id(1));

        assert_eq!(
            state.validate(),
            Err(Error::MaterialValue {
                palette_id: live_palette_id,
                property_id,
                material_id: live_material_id,
            })
        );
    }

    #[test]
    fn remove_object_preserves_the_survivors_order() {
        let mut state = VoxMain::default();
        let a_id = state.add_object(unit_object("a")).unwrap();
        let b_id = state.add_object(unit_object("b")).unwrap();
        let c_id = state.add_object(unit_object("c")).unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing "c" before "b".
        assert_eq!(state.remove_object(a_id), Ok(()));
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "c"]);

        // An object added after the removal recycles the freed id but appends
        // at the end of the order.
        let d_id = state.add_object(unit_object("d")).unwrap();
        assert_eq!(d_id, a_id);
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "c", "d"]);
        assert_eq!(state.object_index(b_id), Some(0));
        assert_eq!(state.object_index(c_id), Some(1));
        assert_eq!(state.object_index(d_id), Some(2));
    }

    #[test]
    fn remove_palette_preserves_the_survivors_order() {
        let mut state = VoxMain::default();
        let a_id = state.add_palette(bare_palette()).unwrap();
        let b_id = state.add_palette(bare_palette()).unwrap();
        let c_id = state.add_palette(bare_palette()).unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing `c_id` before `b_id`.
        assert_eq!(state.remove_palette(a_id), Ok(()));
        assert_eq!(
            state.iter_palettes().map(|(id, _)| id).collect::<Vec<_>>(),
            [b_id, c_id]
        );
    }

    #[test]
    fn remove_palette_detaches_every_layer_drawing_it() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20]);
        let a_id = state.add_palette(two_material_palette(ints_id)).unwrap();
        let b_id = state.add_palette(two_material_palette(ints_id)).unwrap();
        let c_id = state.add_palette(two_material_palette(ints_id)).unwrap();

        // Two of the four layers draw `a_id`, so the detach has to remove
        // both.
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(a_id, material_id(0));
        let on_b_id = object.add_layer(b_id, material_id(0));
        object.add_layer(a_id, material_id(0));
        let on_c_id = object.add_layer(c_id, material_id(0));

        // Each layer samples a different material per voxel, so a detach that
        // drops the wrong sample column shows up below.
        let first_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let second_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();
        object
            .retain_voxel(
                first_id,
                &[
                    material_id(0),
                    material_id(1),
                    material_id(0),
                    material_id(0),
                ],
            )
            .unwrap();
        object
            .retain_voxel(
                second_id,
                &[
                    material_id(1),
                    material_id(0),
                    material_id(1),
                    material_id(1),
                ],
            )
            .unwrap();
        let object_id = state.add_object(object).unwrap();
        state.validate().unwrap();

        assert_eq!(state.remove_palette(a_id), Ok(()));
        state.validate().unwrap();

        // Both layers on `a_id` are gone and the survivors keep their order.
        assert_eq!(
            state.iter_palettes().map(|(id, _)| id).collect::<Vec<_>>(),
            [b_id, c_id]
        );
        let object_ref = state.object(object_id).unwrap();
        assert_eq!(
            object_ref.iter_layers().collect::<Vec<_>>(),
            [(on_b_id, b_id), (on_c_id, c_id)]
        );

        // Each surviving layer kept its own samples.
        assert_eq!(
            object_ref.voxel_material(first_id, on_b_id),
            Some(material_id(1))
        );
        assert_eq!(
            object_ref.voxel_material(second_id, on_b_id),
            Some(material_id(0))
        );
        assert_eq!(
            object_ref.voxel_material(first_id, on_c_id),
            Some(material_id(0))
        );
        assert_eq!(
            object_ref.voxel_material(second_id, on_c_id),
            Some(material_id(1))
        );

        state.gc();
        state.validate().unwrap();
    }

    #[test]
    fn remove_hierarchy_node_preserves_the_survivors_order() {
        let mut state = VoxMain::default();
        let a_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        let b_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();
        let c_id = state
            .add_hierarchy_node(VoxHierarchyNode::default())
            .unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing `c_id` before `b_id`.
        assert_eq!(state.remove_hierarchy_node(a_id), Ok(()));
        assert_eq!(
            state
                .iter_hierarchy_nodes()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [b_id, c_id]
        );
    }

    #[test]
    fn move_object_reorders_the_listing_and_validates() {
        let mut state = VoxMain::default();
        let a_id = state.add_object(unit_object("a")).unwrap();
        let b_id = state.add_object(unit_object("b")).unwrap();
        let c_id = state.add_object(unit_object("c")).unwrap();

        assert_eq!(state.move_object(a_id, 2), Ok(()));
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "c", "a"]);
        assert_eq!(state.object_index(a_id), Some(2));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            state.move_object(a_id, 3),
            Err(Error::IndexPastCount { index: 3, count: 3 })
        );
        state.remove_object(b_id).unwrap();
        assert_eq!(
            state.move_object(b_id, 0),
            Err(Error::UnknownObject { object_id: b_id })
        );
        assert_eq!(state.object_index(b_id), None);
        assert_eq!(state.object_index(c_id), Some(0));
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["c", "a"]);
    }

    #[test]
    fn move_palette_reorders_the_listing_and_validates() {
        let mut state = VoxMain::default();
        let a_id = state.add_palette(bare_palette()).unwrap();
        let b_id = state.add_palette(bare_palette()).unwrap();

        assert_eq!(state.move_palette(b_id, 0), Ok(()));
        assert_eq!(
            state.iter_palettes().map(|(id, _)| id).collect::<Vec<_>>(),
            [b_id, a_id]
        );
        assert_eq!(state.palette_index(b_id), Some(0));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            state.move_palette(b_id, 2),
            Err(Error::IndexPastCount { index: 2, count: 2 })
        );
        assert_eq!(
            state.move_palette(U32Id::from_u32(9), 0),
            Err(Error::UnknownPalette {
                palette_id: palette_id(9)
            })
        );
        assert_eq!(state.palette_index(U32Id::from_u32(9)), None);
    }

    #[test]
    fn move_value_pool_reorders_the_listing_and_validates() {
        let mut state = VoxMain::default();
        let a_id = int_value_pool(&mut state, vec![1]);
        let b_id = int_value_pool(&mut state, vec![2]);

        assert_eq!(state.move_value_pool(b_id, 0), Ok(()));
        assert_eq!(
            state
                .iter_value_pools()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [b_id, a_id]
        );
        assert_eq!(state.value_pool_index(b_id), Some(0));

        // An out-of-range index and an unknown id are rejected.
        assert_eq!(
            state.move_value_pool(b_id, 2),
            Err(Error::IndexPastCount { index: 2, count: 2 })
        );
        assert_eq!(
            state.move_value_pool(U32Id::from_u32(9), 0),
            Err(Error::UnknownValuePool {
                value_pool_id: value_pool_id(9)
            })
        );
        assert_eq!(state.value_pool_index(U32Id::from_u32(9)), None);
    }

    #[test]
    fn remove_value_pool_value_repoints_cells_preserves_order_and_validates() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20, 30]);
        // Two palettes draw the doomed value, so both must be repointed.
        let a = one_material_palette(ints_id, 0);
        let a_id = state.add_palette(a).unwrap();
        let mut b = VoxPalette::default();
        let b_property_id = b
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let b_doomed_id = b.add_material(vec![value_id(0)]).unwrap();
        let b_last_id = b.add_material(vec![value_id(2)]).unwrap();
        let b_id = state.add_palette(b).unwrap();
        state.validate().unwrap();

        // Removing the first of three is the smallest case a swap-remove would
        // get wrong, listing 30 before 20.
        assert_eq!(
            state.remove_value_pool_value(ints_id, value_id(0), value_id(2)),
            Ok(())
        );

        // Every cell that drew 10 now draws 30, and the survivors keep their
        // order and ids.
        let a_property_id = U32Id::<BVoxProperty>::from_u32(0);
        let a_material_id = U32Id::<BVoxMaterial>::from_u32(0);
        assert_eq!(
            state
                .palette(a_id)
                .unwrap()
                .value_id(a_material_id, a_property_id),
            Some(value_id(2))
        );
        let b_ref = state.palette(b_id).unwrap();
        assert_eq!(
            b_ref.value_id(b_doomed_id, b_property_id),
            Some(value_id(2))
        );
        assert_eq!(b_ref.value_id(b_last_id, b_property_id), Some(value_id(2)));
        assert_eq!(
            state.value_pool(ints_id),
            Some(&VoxValuePool::int(VoxBound::None, VoxBound::None, vec![20, 30]).unwrap())
        );
        state.validate().unwrap();

        // A repeated id, an id not the pool's, a released id, and an unknown
        // pool all reject.
        assert_eq!(
            state.remove_value_pool_value(ints_id, value_id(1), value_id(1)),
            Err(Error::SelfReplacement)
        );
        assert_eq!(
            state.remove_value_pool_value(ints_id, value_id(9), value_id(1)),
            Err(Error::UnknownValuePoolValue {
                value_id: value_id(9)
            })
        );
        assert_eq!(
            state.remove_value_pool_value(ints_id, value_id(1), value_id(0)),
            Err(Error::UnknownValuePoolValue {
                value_id: value_id(0)
            })
        );
        assert_eq!(
            state.remove_value_pool_value(U32Id::from_u32(9), value_id(1), value_id(2)),
            Err(Error::UnknownValuePool {
                value_pool_id: value_pool_id(9)
            })
        );
        state.validate().unwrap();
    }

    #[test]
    fn gc_after_moves_renumbers_to_listing_order() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![1, 2]);
        let mut palette = VoxPalette::default();
        let property_id = palette
            .add_property("v".to_owned(), ints_id, value_id(0))
            .unwrap();
        let one_id = palette.add_material(vec![value_id(0)]).unwrap();
        let two_id = palette.add_material(vec![value_id(1)]).unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        let object_a_id = state.add_object(unit_object("a")).unwrap();
        let object_b_id = state.add_object(unit_object("b")).unwrap();
        state.validate().unwrap();

        // List the value holding 2 first and object b first.
        state
            .reorder_value_pool(ints_id, &[value_id(1), value_id(0)])
            .unwrap();
        state.move_object(object_b_id, 0).unwrap();
        state.validate().unwrap();

        let remap = state.gc();
        state.validate().unwrap();

        // Everything renumbers to listing order: the value holding 2 is now id
        // 0, object b is id 0, and every id equals its listing index.
        let names: Vec<&str> = state.iter_objects().map(|(_, o)| o.name()).collect();
        assert_eq!(names, ["b", "a"]);
        assert_eq!(remap.objects.new_id(object_b_id), Some(U32Id::from_u32(0)));
        assert_eq!(remap.objects.new_id(object_a_id), Some(U32Id::from_u32(1)));
        assert_eq!(
            state
                .iter_objects()
                .map(|(id, _)| id.to_u32())
                .collect::<Vec<_>>(),
            [0, 1]
        );

        // The value remap is indexed by the pool's old id. The value holding 2
        // moved from id 1 to id 0.
        assert_eq!(
            remap.value_pool_values[ints_id.to_usize_id()].new_id(value_id(1)),
            Some(value_id(0))
        );
        assert_eq!(
            remap.value_pool_values[ints_id.to_usize_id()].new_id(value_id(0)),
            Some(value_id(1))
        );
        let value_pool = state.value_pool(ints_id).unwrap();
        assert_eq!(
            value_pool.value(value_id(0)),
            Some(VoxValuePoolValueRef::Int(2))
        );
        assert_eq!(
            value_pool.value(value_id(1)),
            Some(VoxValuePoolValueRef::Int(1))
        );
        assert_eq!(
            value_pool
                .iter_values()
                .map(|(id, _)| id.to_u32())
                .collect::<Vec<_>>(),
            [0, 1]
        );

        // The material cells followed the value renumbering, so each still
        // resolves to its number.
        let palette_ref = state.palette(palette_id).unwrap();
        assert_eq!(palette_ref.value_id(one_id, property_id), Some(value_id(1)));
        assert_eq!(palette_ref.value_id(two_id, property_id), Some(value_id(0)));
    }

    #[test]
    fn gc_after_a_value_pool_move_relabels_value_pools_and_each_value_pool_s_values() {
        let mut state = VoxMain::default();
        let first_value_pool_id = int_value_pool(&mut state, vec![10, 20]);
        let second_value_pool_id = int_value_pool(&mut state, vec![30, 40, 50]);
        let mut palette = VoxPalette::default();
        // Both properties come before the material, so neither is back-filled.
        let first_id = palette
            .add_property("first".to_owned(), first_value_pool_id, value_id(0))
            .unwrap();
        let second_id = palette
            .add_property("second".to_owned(), second_value_pool_id, value_id(0))
            .unwrap();
        let live_material_id = palette
            .add_material(vec![value_id(1), value_id(0)])
            .unwrap();
        let palette_id = state.add_palette(palette).unwrap();
        state.validate().unwrap();

        // Move the second pool ahead of the first, so the pool relabel is not
        // the identity, and give the two pools different value permutations, so
        // a cell relabeled through the wrong pool's remap lands on the wrong
        // value.
        state.move_value_pool(second_value_pool_id, 0).unwrap();
        state
            .reorder_value_pool(first_value_pool_id, &[value_id(1), value_id(0)])
            .unwrap();
        state
            .reorder_value_pool(
                second_value_pool_id,
                &[value_id(2), value_id(0), value_id(1)],
            )
            .unwrap();
        state.validate().unwrap();

        let remap = state.gc();
        state.validate().unwrap();

        // The pools renumber to listing order, so the moved pool is now id 0.
        assert_eq!(
            remap.value_pools.new_id(second_value_pool_id),
            Some(value_pool_id(0))
        );
        assert_eq!(
            remap.value_pools.new_id(first_value_pool_id),
            Some(value_pool_id(1))
        );
        assert_eq!(
            state
                .iter_value_pools()
                .map(|(id, _)| id.to_u32())
                .collect::<Vec<_>>(),
            [0, 1]
        );

        // Each pool's value remap is keyed by that pool's pre-gc id, which is
        // the id the moved pool held before the relabel above.
        assert_eq!(
            remap.value_pool_values[first_value_pool_id.to_usize_id()].new_id(value_id(1)),
            Some(value_id(0))
        );
        assert_eq!(
            remap.value_pool_values[second_value_pool_id.to_usize_id()].new_id(value_id(2)),
            Some(value_id(0))
        );

        // Every property followed the pool renumbering.
        let palette_ref = state.palette(palette_id).unwrap();
        assert_eq!(
            palette_ref.property(first_id).unwrap().value_pool_id,
            value_pool_id(1)
        );
        assert_eq!(
            palette_ref.property(second_id).unwrap().value_pool_id,
            value_pool_id(0)
        );

        // The material still reads the same two numbers, through the relabeled
        // pool ids and the relabeled cells.
        let (value_pool_ref, resolved_value_id) = state
            .material_value(palette_id, live_material_id, first_id)
            .unwrap();
        assert_eq!(
            value_pool_ref.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Int(20))
        );
        let (value_pool_ref, resolved_value_id) = state
            .material_value(palette_id, live_material_id, second_id)
            .unwrap();
        assert_eq!(
            value_pool_ref.value(resolved_value_id),
            Some(VoxValuePoolValueRef::Int(30))
        );
    }

    #[test]
    fn object_methods_edit_an_inserted_object() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![1, 2]);
        let palette_id = state.add_palette(two_material_palette(ints_id)).unwrap();
        let object_id = state.add_object(unit_object("o")).unwrap();

        // add_layer back-fills the live voxel with the default material.
        let base_id = state
            .add_layer(object_id, palette_id, material_id(0))
            .unwrap();
        assert_eq!(
            state
                .object(object_id)
                .unwrap()
                .voxel_material(voxel_id(0), base_id),
            Some(material_id(0))
        );

        // retain_voxel swaps the sample; a material beyond the layer's
        // palette is rejected.
        state
            .retain_voxel(object_id, voxel_id(0), &[material_id(1)])
            .unwrap();
        assert_eq!(
            state
                .object(object_id)
                .unwrap()
                .voxel_material(voxel_id(0), base_id),
            Some(material_id(1))
        );
        assert_eq!(
            state.retain_voxel(object_id, voxel_id(0), &[material_id(9)]),
            Err(Error::LayerSampleMaterial {
                layer_id: base_id,
                voxel_id: voxel_id(0),
                material_id: material_id(9),
            })
        );

        // The remaining methods address the object by id.
        let overlay_id = state
            .add_layer(object_id, palette_id, material_id(1))
            .unwrap();
        state.move_layer(object_id, overlay_id, 0).unwrap();
        assert_eq!(
            state
                .object(object_id)
                .unwrap()
                .iter_layers()
                .map(|(id, _)| id)
                .collect::<Vec<_>>(),
            [overlay_id, base_id]
        );
        state.remove_layer(object_id, overlay_id).unwrap();
        state
            .set_object_origin(object_id, TyVector3I32::new(1, 2, 3))
            .unwrap();
        assert_eq!(
            state.object(object_id).unwrap().origin(),
            TyVector3I32::new(1, 2, 3)
        );
        state.release_voxel(object_id, voxel_id(0)).unwrap();
        assert_eq!(state.object(object_id).unwrap().live_count(), 0);
        state.validate().unwrap();
    }

    #[test]
    fn object_methods_reject_bad_ids() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![1, 2]);
        let live_palette_id = state.add_palette(two_material_palette(ints_id)).unwrap();
        let object_id = state.add_object(unit_object("o")).unwrap();

        let ghost_id = U32Id::<BVoxObject>::from_u32(9);
        assert_eq!(
            state.add_layer(ghost_id, live_palette_id, material_id(0)),
            Err(Error::UnknownObject {
                object_id: ghost_id
            })
        );
        assert_eq!(
            state.add_layer(object_id, palette_id(9), material_id(0)),
            Err(Error::UnknownPalette {
                palette_id: palette_id(9)
            })
        );
        assert_eq!(
            state.add_layer(object_id, live_palette_id, material_id(9)),
            Err(Error::UnknownMaterial {
                material_id: material_id(9)
            })
        );
        assert_eq!(
            state.retain_voxel(object_id, voxel_id(9), &[]),
            Err(Error::UnknownVoxel {
                voxel_id: voxel_id(9)
            })
        );
        assert_eq!(
            state.retain_voxel(object_id, voxel_id(0), &[material_id(0)]),
            Err(Error::SampleArity {
                samples: 1,
                layers: 0
            })
        );
        assert_eq!(
            state.remove_layer(object_id, U32Id::<BVoxLayer>::from_u32(9)),
            Err(Error::UnknownLayer {
                layer_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(state.object(object_id).unwrap().layer_count(), 0);
        state.validate().unwrap();
    }

    #[test]
    fn palette_methods_edit_an_inserted_palette() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20]);
        let palette_id = state.add_palette(one_material_palette(ints_id, 0)).unwrap();

        // add_property back-fills the existing material with the default.
        let tag_id = state
            .add_property(palette_id, "tag".to_owned(), ints_id, value_id(1))
            .unwrap();
        let first_id = state
            .palette(palette_id)
            .unwrap()
            .iter_materials()
            .next()
            .unwrap();
        assert_eq!(
            state
                .palette(palette_id)
                .unwrap()
                .value_id(first_id, tag_id),
            Some(value_id(1))
        );

        // add_material takes one value id per property, each within its pool.
        let second_id = state
            .add_material(palette_id, vec![value_id(1), value_id(0)])
            .unwrap();
        assert_eq!(
            state
                .palette(palette_id)
                .unwrap()
                .value_id(second_id, tag_id),
            Some(value_id(0))
        );

        state.remove_property(palette_id, tag_id).unwrap();
        assert_eq!(state.palette(palette_id).unwrap().property_count(), 1);
        state.validate().unwrap();
    }

    #[test]
    fn palette_methods_reject_bad_ids() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20]);
        let palette_id = state.add_palette(one_material_palette(ints_id, 0)).unwrap();

        let ghost_id = U32Id::<BVoxPalette>::from_u32(9);
        assert_eq!(
            state.add_property(ghost_id, "tag".to_owned(), ints_id, value_id(0)),
            Err(Error::UnknownPalette {
                palette_id: ghost_id
            })
        );
        assert_eq!(
            state.add_property(palette_id, "tag".to_owned(), value_pool_id(9), value_id(0)),
            Err(Error::UnknownValuePool {
                value_pool_id: value_pool_id(9)
            })
        );
        assert_eq!(
            state.add_property(palette_id, "tag".to_owned(), ints_id, value_id(9)),
            Err(Error::UnknownValuePoolValue {
                value_id: value_id(9)
            })
        );
        assert_eq!(
            state.add_property(palette_id, "v".to_owned(), ints_id, value_id(0)),
            Err(Error::DuplicatePropertyName {
                name: "v".to_owned()
            })
        );

        // A wrong arity and a value outside the property's pool are rejected.
        assert_eq!(
            state.add_material(palette_id, vec![]),
            Err(Error::MaterialValueArity {
                values: 0,
                properties: 1
            })
        );
        assert_eq!(
            state.add_material(palette_id, vec![value_id(9)]),
            Err(Error::UnknownValuePoolValue {
                value_id: value_id(9)
            })
        );

        assert_eq!(
            state.remove_property(palette_id, U32Id::<BVoxProperty>::from_u32(9)),
            Err(Error::UnknownProperty {
                property_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(state.palette(palette_id).unwrap().material_count(), 1);
        state.validate().unwrap();
    }

    #[test]
    fn add_hierarchy_node_rejects_a_dangling_child_object() {
        let mut state = VoxMain::default();
        assert_eq!(
            state.add_hierarchy_node(node_with_objects(vec![U32Id::from_u32(9)])),
            Err(Error::UnknownObject {
                object_id: U32Id::from_u32(9)
            })
        );
        assert_eq!(state.hierarchy_node_count(), 0);
    }

    #[test]
    fn remove_material_cannot_empty_a_palette() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10]);
        let palette_id = state.add_palette(one_material_palette(ints_id, 0)).unwrap();
        let only_id = state
            .palette(palette_id)
            .unwrap()
            .iter_materials()
            .next()
            .unwrap();

        // The removal needs a distinct live replacement, which a one-material
        // palette cannot supply.
        assert_eq!(
            state.remove_material(palette_id, only_id, only_id),
            Err(Error::SelfReplacement)
        );
        assert_eq!(
            state.remove_material(palette_id, only_id, material_id(1)),
            Err(Error::UnknownMaterial {
                material_id: material_id(1)
            })
        );
        assert_eq!(state.palette(palette_id).unwrap().material_count(), 1);
    }

    /// Everything the readers expose, rendered so a half-applied mutation
    /// inside an entity shows up. The state's own debug rendering carries the
    /// id pools, the roots, and the ext, and stops at each entity's edge.
    fn snapshot(state: &VoxMain) -> String {
        let mut out = format!("{state:?}");
        for (value_pool_id, value_pool) in state.iter_value_pools() {
            let values: Vec<_> = value_pool.iter_values().collect();
            out += &format!(
                "|value pool {value_pool_id:?} {:?} {values:?}",
                value_pool.kind()
            );
        }
        for (palette_id, palette) in state.iter_palettes() {
            let properties: Vec<_> = palette.iter_properties().collect();
            out += &format!("|palette {palette_id:?} {properties:?}");
            for material_id in palette.iter_materials() {
                let row: Vec<_> = palette
                    .iter_properties()
                    .map(|(property_id, _)| palette.value_id(material_id, property_id))
                    .collect();
                out += &format!("|material {material_id:?} {row:?}");
            }
        }
        for (object_id, object) in state.iter_objects() {
            let live: Vec<_> = object.iter_live().collect();
            out += &format!(
                "|object {object_id:?} {} {:?} {:?} {live:?}",
                object.name(),
                object.bounds(),
                object.origin()
            );
            for (layer_id, palette_id) in object.iter_layers() {
                let samples: Vec<_> = object
                    .iter_live_samples(layer_id)
                    .expect("an iterated layer is one of the object's")
                    .collect();
                out += &format!("|layer {layer_id:?} {palette_id:?} {samples:?}");
            }
        }
        for (node_id, node) in state.iter_hierarchy_nodes() {
            out += &format!("|node {node_id:?} {node:?}");
        }
        out
    }

    /// Applies a mutation expected to fail and asserts every reader-visible
    /// value is byte-for-byte unchanged.
    fn assert_rejects_unchanged<T>(
        state: &mut VoxMain,
        mutate: impl FnOnce(&mut VoxMain) -> Result<T>,
    ) {
        let before = snapshot(state);
        assert!(mutate(state).is_err());
        assert_eq!(snapshot(state), before);
    }

    #[test]
    fn rejected_mutations_change_nothing() {
        let mut state = VoxMain::default();
        let ints_id = int_value_pool(&mut state, vec![10, 20]);
        let live_palette_id = state.add_palette(two_material_palette(ints_id)).unwrap();
        let object_id = state.add_object(unit_object("o")).unwrap();
        let layer_id = state
            .add_layer(object_id, live_palette_id, material_id(0))
            .unwrap();
        let live_node_id = state
            .add_hierarchy_node(node_with_objects(vec![object_id]))
            .unwrap();
        state.push_root_hierarchy_node_id(live_node_id).unwrap();
        state
            .retain_voxel(object_id, voxel_id(0), &[material_id(1)])
            .unwrap();
        state.validate().unwrap();

        // Insertions.
        assert_rejects_unchanged(&mut state, |s| s.add_palette(VoxPalette::default()));
        assert_rejects_unchanged(&mut state, |s| {
            s.add_palette(one_material_palette(value_pool_id(9), 0))
        });
        assert_rejects_unchanged(&mut state, |s| {
            let mut bad = unit_object("bad");
            bad.add_layer(palette_id(9), material_id(0));
            s.add_object(bad)
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.add_hierarchy_node(node_with_children(vec![node_id(9)]))
        });
        assert_rejects_unchanged(&mut state, |s| {
            // The two batch nodes reference each other by prospective id.
            s.add_hierarchy_nodes(vec![
                node_with_children(vec![node_id(2)]),
                node_with_children(vec![node_id(1)]),
            ])
        });

        // Root setters.
        assert_rejects_unchanged(&mut state, |s| s.push_root_hierarchy_node_id(node_id(9)));
        assert_rejects_unchanged(&mut state, |s| s.push_root_hierarchy_node_id(live_node_id));
        assert_rejects_unchanged(&mut state, |s| {
            s.set_root_hierarchy_node_ids(vec![live_node_id, live_node_id])
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.set_root_hierarchy_node_ids(vec![node_id(9)])
        });

        // Moves and reorders.
        assert_rejects_unchanged(&mut state, |s| s.move_object(object_id, 1));
        assert_rejects_unchanged(&mut state, |s| s.move_palette(palette_id(9), 0));
        assert_rejects_unchanged(&mut state, |s| s.move_value_pool(ints_id, 1));
        assert_rejects_unchanged(&mut state, |s| {
            s.reorder_value_pool(ints_id, &[value_id(0)])
        });

        // Object edits.
        assert_rejects_unchanged(&mut state, |s| {
            s.add_layer(object_id, palette_id(9), material_id(0))
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.add_layer(object_id, live_palette_id, material_id(9))
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.retain_voxel(object_id, voxel_id(0), &[material_id(9)])
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.retain_voxel(object_id, voxel_id(9), &[material_id(0)])
        });
        assert_rejects_unchanged(&mut state, |s| s.retain_voxel(object_id, voxel_id(0), &[]));
        assert_rejects_unchanged(&mut state, |s| s.release_voxel(object_id, voxel_id(9)));
        assert_rejects_unchanged(&mut state, |s| {
            s.remove_layer(object_id, U32Id::from_u32(9))
        });
        assert_rejects_unchanged(&mut state, |s| s.move_layer(object_id, layer_id, 1));
        assert_rejects_unchanged(&mut state, |s| {
            s.set_object_origin(U32Id::from_u32(9), TyVector3I32::new(0, 0, 0))
        });

        // Palette edits.
        assert_rejects_unchanged(&mut state, |s| {
            s.add_property(live_palette_id, "v".to_owned(), ints_id, value_id(0))
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.add_property(live_palette_id, "w".to_owned(), ints_id, value_id(9))
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.add_material(live_palette_id, vec![value_id(9)])
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.remove_property(live_palette_id, U32Id::from_u32(9))
        });

        // Removals.
        assert_rejects_unchanged(&mut state, |s| s.remove_object(U32Id::from_u32(9)));
        assert_rejects_unchanged(&mut state, |s| s.remove_palette(palette_id(9)));
        assert_rejects_unchanged(&mut state, |s| s.remove_hierarchy_node(node_id(9)));
        assert_rejects_unchanged(&mut state, |s| {
            s.remove_material(live_palette_id, material_id(0), material_id(0))
        });
        assert_rejects_unchanged(&mut state, |s| {
            s.remove_value_pool_value(ints_id, value_id(0), value_id(0))
        });

        state.validate().unwrap();
    }

    /// A deterministic linear-congruential generator, so the operation
    /// sequence needs no randomness dependency and replays exactly.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }

        /// A value in `[0, bound)`.
        fn below(&mut self, bound: usize) -> usize {
            (self.next() % bound as u64) as usize
        }
    }

    /// Applies one operation drawn from `rng`. Ids come from a small range so
    /// calls hit live and dead entities alike, and every `Result` is dropped:
    /// the property under test is that no success breaks the state.
    fn apply_random_operation(state: &mut VoxMain, rng: &mut Lcg) {
        let wild_value_pool_id = value_pool_id(rng.below(6) as u32);
        let wild_palette_id = palette_id(rng.below(6) as u32);
        let wild_object_id = U32Id::<BVoxObject>::from_u32(rng.below(6) as u32);
        let wild_node_id = node_id(rng.below(8) as u32);
        let wild_material_id = material_id(rng.below(4) as u32);
        let wild_value_id = value_id(rng.below(4) as u32);
        match rng.below(21) {
            0 => {
                let values = (0..1 + rng.below(3)).map(|v| v as i64).collect();
                state.add_value_pool(
                    VoxValuePool::int(VoxBound::None, VoxBound::None, values).unwrap(),
                );
            }
            1 => {
                let mut palette = VoxPalette::default();
                for index in 0..rng.below(3) {
                    let _ = palette.add_property(
                        format!("p{index}"),
                        wild_value_pool_id,
                        wild_value_id,
                    );
                }
                for _ in 0..1 + rng.below(2) {
                    let row = (0..palette.property_count())
                        .map(|_| value_id(rng.below(4) as u32))
                        .collect();
                    let _ = palette.add_material(row);
                }
                let _ = state.add_palette(palette);
            }
            2 => {
                let bounds = TyVector3U32::new(1 + rng.below(2) as u32, 1 + rng.below(2) as u32, 1);
                let mut object = VoxObject::new(String::new(), bounds).unwrap();
                if rng.below(2) == 0 {
                    object.add_layer(wild_palette_id, wild_material_id);
                }
                let sample_ids: Vec<_> = (0..object.layer_count())
                    .map(|_| material_id(rng.below(4) as u32))
                    .collect();
                let _ = object.retain_voxel(voxel_id(rng.below(4) as u32), &sample_ids);
                let _ = state.add_object(object);
            }
            3 => {
                let _ = state.add_hierarchy_node(VoxHierarchyNode {
                    child_node_ids: (0..rng.below(3))
                        .map(|_| node_id(rng.below(8) as u32))
                        .collect(),
                    child_object_ids: (0..rng.below(2))
                        .map(|_| U32Id::from_u32(rng.below(6) as u32))
                        .collect(),
                    ..VoxHierarchyNode::default()
                });
            }
            4 => {
                let nodes = (0..1 + rng.below(3))
                    .map(|_| {
                        node_with_children(
                            (0..rng.below(3))
                                .map(|_| node_id(rng.below(10) as u32))
                                .collect(),
                        )
                    })
                    .collect();
                let _ = state.add_hierarchy_nodes(nodes);
            }
            5 => {
                let _ = state.push_root_hierarchy_node_id(wild_node_id);
            }
            6 => {
                let root_ids = (0..rng.below(3))
                    .map(|_| node_id(rng.below(8) as u32))
                    .collect();
                let _ = state.set_root_hierarchy_node_ids(root_ids);
            }
            7 => {
                let _ = state.add_layer(wild_object_id, wild_palette_id, wild_material_id);
            }
            8 => {
                let layers = state
                    .object(wild_object_id)
                    .map_or(0, VoxObject::layer_count);
                let sample_ids: Vec<_> = (0..layers)
                    .map(|_| material_id(rng.below(4) as u32))
                    .collect();
                let _ =
                    state.retain_voxel(wild_object_id, voxel_id(rng.below(6) as u32), &sample_ids);
            }
            9 => {
                let _ = state.release_voxel(wild_object_id, voxel_id(rng.below(6) as u32));
            }
            10 => {
                let _ = state.remove_layer(wild_object_id, U32Id::from_u32(rng.below(3) as u32));
            }
            11 => {
                let _ = state.move_layer(
                    wild_object_id,
                    U32Id::from_u32(rng.below(3) as u32),
                    rng.below(3),
                );
            }
            12 => {
                let _ = state.add_property(
                    wild_palette_id,
                    format!("p{}", rng.below(4)),
                    wild_value_pool_id,
                    wild_value_id,
                );
            }
            13 => {
                let arity = state
                    .palette(wild_palette_id)
                    .map_or(0, VoxPalette::property_count);
                let row = (0..arity).map(|_| value_id(rng.below(4) as u32)).collect();
                let _ = state.add_material(wild_palette_id, row);
            }
            14 => {
                let _ =
                    state.remove_property(wild_palette_id, U32Id::from_u32(rng.below(3) as u32));
            }
            15 => {
                let _ = state.remove_object(wild_object_id);
            }
            16 => {
                let _ = state.remove_palette(wild_palette_id);
            }
            17 => {
                let _ = state.remove_hierarchy_node(wild_node_id);
            }
            18 => {
                let _ = state.remove_material(
                    wild_palette_id,
                    wild_material_id,
                    material_id(rng.below(4) as u32),
                );
            }
            19 => {
                let _ = state.remove_value_pool_value(
                    wild_value_pool_id,
                    wild_value_id,
                    value_id(rng.below(4) as u32),
                );
            }
            _ => {
                let _ = state.move_object(wild_object_id, rng.below(4));
            }
        }
    }

    // Miri interprets the long operation sequence far too slowly; the unsafe
    // paths it exercises are covered by the focused tests above.
    /// The payoff property: whatever mix of successes and rejections a
    /// sequence of safe-API calls produces, the state always audits clean.
    #[test]
    #[cfg_attr(miri, ignore)]
    fn a_seeded_operation_sequence_keeps_the_state_valid() {
        for seed in 0..3 {
            let mut rng = Lcg(seed);
            let mut state = VoxMain::default();
            for step in 0..600 {
                apply_random_operation(&mut state, &mut rng);
                if step % 60 == 0 {
                    assert_eq!(state.validate(), Ok(()), "seed {seed} step {step}");
                }
            }
            assert_eq!(state.validate(), Ok(()), "seed {seed} before gc");
            state.gc();
            assert_eq!(state.validate(), Ok(()), "seed {seed} after gc");
        }
    }
}
