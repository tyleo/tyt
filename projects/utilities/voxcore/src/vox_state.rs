use crate::{
    BVoxEffectiveProperty, BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty,
    BVoxValuePool, BVoxValuePoolValue, Error, Result, VoxEffectivePalette, VoxEffectiveProperty,
    VoxHierarchyNode, VoxObject, VoxPalette, VoxValuePool,
};
use branded_id::{
    IdVec, U32Id, UsizeId,
    soa::{IdField, IdStruct},
};
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionExt, UNIT_ROTATION_TOLERANCE};

/// The scene of a voxel model: the read side of [`VoxMain`](crate::VoxMain),
/// which forwards to it.
///
/// This is the struct-of-arrays backing store. [`VoxMain`](crate::VoxMain) owns
/// mutation logic over these fields; they are crate-private so the id pools and
/// columns stay in sync. An ext hook reads the scene through this type.
#[derive(Debug, Default)]
pub struct VoxState {
    /// Value-pool id pool.
    pub(crate) value_pool_ids: IdStruct<BVoxValuePool>,

    /// The shared value pools.
    pub(crate) value_pools: IdField<BVoxValuePool, VoxValuePool>,

    /// Palette id pool.
    pub(crate) palette_ids: IdStruct<BVoxPalette>,

    /// The shared palettes.
    pub(crate) palettes: IdField<BVoxPalette, VoxPalette>,

    /// Object id pool.
    pub(crate) object_ids: IdStruct<BVoxObject>,

    /// The objects.
    pub(crate) objects: IdField<BVoxObject, VoxObject>,

    /// Hierarchy node id pool.
    pub(crate) hierarchy_node_ids: IdStruct<BVoxHierarchyNode>,

    /// The hierarchy nodes.
    pub(crate) hierarchy_nodes: IdField<BVoxHierarchyNode, VoxHierarchyNode>,

    /// The scene's roots: hierarchy node ids.
    pub(crate) root_hierarchy_node_ids: Vec<U32Id<BVoxHierarchyNode>>,
}

impl Drop for VoxState {
    fn drop(&mut self) {
        // Safety: each column holds a value for every id in its id pool; the
        // fields free their own storage on drop.
        unsafe {
            self.value_pools.release_all(&self.value_pool_ids);
            self.palettes.release_all(&self.palette_ids);
            self.objects.release_all(&self.object_ids);
            self.hierarchy_nodes.release_all(&self.hierarchy_node_ids);
        }
    }
}

impl VoxState {
    /// Audits the full rule set. Every rule here is also enforced at a mutation
    /// point (a constructor, an insertion, or the mutation itself), so a state
    /// reached through the public API always passes; a failure reports a
    /// voxcore bug, never a caller error. The checks stay as the specification
    /// of what the mutations preserve:
    ///
    /// 1. every value pool value is within its kind's value domain
    /// 2. per palette:
    ///    1. every property names a live value pool
    ///    2. no property name repeats
    ///    3. every material value id is within its property's value pool
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
    /// A node may have several parents because the hierarchy is a DAG; that
    /// sharing is not a cycle.
    pub fn validate(&self) -> Result<()> {
        // Value pool values are within their kind's value domain. This runs
        // first, so a palette that reads a malformed value pool is reported
        // after the value pool it reads.
        for (value_pool_id, value_pool) in self.iter_value_pools() {
            if let Some(value_id) = value_pool.first_out_of_domain_value() {
                return Err(Error::ValuePoolValue {
                    value_pool_id,
                    value_id,
                });
            }
        }

        // Palette property rules: value pools resolve, names are unique, and
        // every value id is within its value pool.
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
        }

        // Object layer palette refs and live-voxel sample materials. Checks are
        // by id retention, not index range, so they hold whether or not
        // releases have left the id pools with holes. Because two layers may
        // reference the same palette, there is no duplicate-layer rule.
        for (object_id, object) in self.iter_objects() {
            let mut layer_palettes = Vec::with_capacity(object.layer_count());
            for (layer_id, palette_id) in object.iter_layers() {
                let palette = self.palette(palette_id).ok_or(Error::PaletteRef {
                    object_id,
                    palette_id,
                })?;
                layer_palettes.push((layer_id, palette));
            }

            // Every live voxel samples a material within each layer's palette.
            // Layer-major so each layer's sample column is read once.
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
            if !position.is_finite() || !scale.is_finite() {
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
        let mut seen_root_ids = HashSet::with_capacity(self.root_hierarchy_node_ids.len());

        for &root_id in &self.root_hierarchy_node_ids {
            if self.hierarchy_node(root_id).is_none() {
                return Err(Error::Root { root_id });
            }
            if !seen_root_ids.insert(root_id) {
                return Err(Error::DuplicateRoot { root_id });
            }
        }

        // Acyclicity; every child is now known live. Works over the retained
        // node ids by position, so it holds whether or not the node id pool has
        // holes.
        let node_ids: Vec<_> = self.hierarchy_node_ids.iter().collect();
        let index_of: HashMap<U32Id<BVoxHierarchyNode>, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(node_index, &node_id)| (node_id, node_index))
            .collect();

        let children: Vec<&[U32Id<BVoxHierarchyNode>]> = node_ids
            .iter()
            .map(|&node_id| {
                // Safety: `node_id` is a retained node id.
                unsafe { self.hierarchy_nodes.get(node_id) }
                    .child_node_ids
                    .as_slice()
            })
            .collect();

        if let Some(node_index) = first_cycle_node_index(&children, &index_of) {
            return Err(Error::Cycle {
                node_id: node_ids[node_index],
            });
        }

        Ok(())
    }

    /// Checks a node about to be inserted at listing position `node_index` of
    /// its batch, resolving child nodes against this state and the batch's
    /// prospective `batch_ids`.
    pub(crate) fn check_inserted_node(
        &self,
        node: &VoxHierarchyNode,
        node_index: usize,
        batch_ids: &HashSet<U32Id<BVoxHierarchyNode>>,
    ) -> Result<()> {
        let mut seen_child_node_ids = HashSet::with_capacity(node.child_node_ids.len());

        for &child_id in &node.child_node_ids {
            if self.hierarchy_node(child_id).is_none() && !batch_ids.contains(&child_id) {
                return Err(Error::UnknownHierarchyNode { node_id: child_id });
            }

            if !seen_child_node_ids.insert(child_id) {
                return Err(Error::InsertedDuplicateChildNode {
                    index: node_index,
                    child_id,
                });
            }
        }

        let mut seen_child_object_ids = HashSet::with_capacity(node.child_object_ids.len());

        for &object_id in &node.child_object_ids {
            if self.object(object_id).is_none() {
                return Err(Error::UnknownObject { object_id });
            }

            if !seen_child_object_ids.insert(object_id) {
                return Err(Error::InsertedDuplicateChildObject {
                    index: node_index,
                    object_id,
                });
            }
        }

        // The rotation needs no finiteness guard of its own: a non-finite
        // component fails the unit-length check.
        let position = node.transform.position;
        let scale = node.transform.scale;
        if !position.is_finite() || !scale.is_finite() {
            return Err(Error::InsertedNonFiniteTransform { index: node_index });
        }

        if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
            return Err(Error::InsertedZeroScale { index: node_index });
        }

        if !node
            .transform
            .rotation
            .is_normalized_within(UNIT_ROTATION_TOLERANCE)
        {
            return Err(Error::InsertedNonUnitRotation { index: node_index });
        }

        Ok(())
    }

    /// The hierarchy node `id`, or `None` if not one of this state's.
    pub fn hierarchy_node(&self, id: U32Id<BVoxHierarchyNode>) -> Option<&VoxHierarchyNode> {
        // Safety: retained ids have a value.
        self.hierarchy_node_ids
            .is_retained(id)
            .then(|| unsafe { self.hierarchy_nodes.get(id) })
    }

    /// Number of hierarchy nodes.
    pub fn hierarchy_node_count(&self) -> usize {
        self.hierarchy_node_ids.len()
    }

    /// Hierarchy nodes in listing order, as `(id, node)`.
    pub fn iter_hierarchy_nodes(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxHierarchyNode>, &VoxHierarchyNode)> + '_ {
        // Safety: retained ids have a value.
        self.hierarchy_node_ids
            .iter()
            .map(move |node_id| (node_id, unsafe { self.hierarchy_nodes.get(node_id) }))
    }

    /// Whether `target` is one of `from` or reachable from any of them through
    /// `child_node_ids`. The walk is iterative, so a deep chain cannot
    /// overflow the stack, and visits a shared node once.
    pub(crate) fn reaches_hierarchy_node(
        &self,
        from: &[U32Id<BVoxHierarchyNode>],
        target: U32Id<BVoxHierarchyNode>,
    ) -> bool {
        let mut visited = HashSet::new();
        let mut stack: Vec<U32Id<BVoxHierarchyNode>> = from.to_vec();

        while let Some(node_id) = stack.pop() {
            if node_id == target {
                return true;
            }

            if !visited.insert(node_id) {
                continue;
            }

            if let Some(node) = self.hierarchy_node(node_id) {
                stack.extend(node.child_node_ids.iter().copied());
            }
        }

        false
    }

    /// Resolves what `material_id` in `palette_id` draws for `property_id`: the
    /// value pool the property draws from and the value id it holds in that
    /// value pool. `None` if any id is not this state's, `property_id` is not
    /// `palette_id`'s, or the property names a value pool this state does not
    /// hold. Read the value at that id out of the returned value pool by the
    /// value pool's kind.
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

    /// Objects in listing order, as `(id, object)`.
    pub fn iter_objects(&self) -> impl Iterator<Item = (U32Id<BVoxObject>, &VoxObject)> + '_ {
        // Safety: retained ids have a value.
        self.object_ids
            .iter()
            .map(move |object_id| (object_id, unsafe { self.objects.get(object_id) }))
    }

    /// The object `id`, or `None` if not one of this state's.
    pub fn object(&self, id: U32Id<BVoxObject>) -> Option<&VoxObject> {
        // Safety: retained ids have a value.
        self.object_ids
            .is_retained(id)
            .then(|| unsafe { self.objects.get(id) })
    }

    /// Number of objects.
    pub fn object_count(&self) -> usize {
        self.object_ids.len()
    }

    /// The effective palette of `object`, resolving its layer override rule
    /// once. Layers are walked front to back, each palette property landing at
    /// its name's entry, so the last supplying layer wins while the first fixes
    /// the entry's position. Errors if a layer references a palette that is not
    /// one of this state's.
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

    /// Palettes in listing order, as `(id, palette)`.
    pub fn iter_palettes(&self) -> impl Iterator<Item = (U32Id<BVoxPalette>, &VoxPalette)> + '_ {
        // Safety: retained ids have a value.
        self.palette_ids
            .iter()
            .map(move |palette_id| (palette_id, unsafe { self.palettes.get(palette_id) }))
    }

    /// The palette `id`, or `None` if not one of this state's.
    pub fn palette(&self, id: U32Id<BVoxPalette>) -> Option<&VoxPalette> {
        // Safety: retained ids have a value.
        self.palette_ids
            .is_retained(id)
            .then(|| unsafe { self.palettes.get(id) })
    }

    /// Number of shared palettes.
    pub fn palette_count(&self) -> usize {
        self.palette_ids.len()
    }

    /// The scene's roots: hierarchy node ids.
    pub fn root_hierarchy_node_ids(&self) -> &[U32Id<BVoxHierarchyNode>] {
        &self.root_hierarchy_node_ids
    }

    /// Value pools in listing order, as `(id, value_pool)`.
    pub fn iter_value_pools(
        &self,
    ) -> impl Iterator<Item = (U32Id<BVoxValuePool>, &VoxValuePool)> + '_ {
        // Safety: retained ids have a value.
        self.value_pool_ids.iter().map(move |value_pool_id| {
            (value_pool_id, unsafe {
                self.value_pools.get(value_pool_id)
            })
        })
    }

    /// The value pool `id`, or `None` if not one of this state's.
    pub fn value_pool(&self, id: U32Id<BVoxValuePool>) -> Option<&VoxValuePool> {
        // Safety: retained ids have a value.
        self.value_pool_ids
            .is_retained(id)
            .then(|| unsafe { self.value_pools.get(id) })
    }

    /// Number of shared value pools.
    pub fn value_pool_count(&self) -> usize {
        self.value_pool_ids.len()
    }
}

/// The `children` index of a node lying on a `child_node_ids` cycle, or `None`
/// if the graph is acyclic.
///
/// `children` holds each node's child ids at that node's index, and `index_of`
/// maps a child id back to its index. A child missing from `index_of` leads
/// outside the checked set, where no edge can return, so it is skipped.
///
/// The walk is an iterative three-colour DFS, so a deep chain cannot overflow
/// the stack. A back edge into an in-progress node is a cycle; revisiting a
/// finished one is not.
pub(crate) fn first_cycle_node_index(
    children: &[&[U32Id<BVoxHierarchyNode>]],
    index_of: &HashMap<U32Id<BVoxHierarchyNode>, usize>,
) -> Option<usize> {
    const WHITE: u8 = 0;
    const GREY: u8 = 1;
    const BLACK: u8 = 2;

    let count = children.len();
    let mut colour = vec![WHITE; count];

    for start_index in 0..count {
        if colour[start_index] != WHITE {
            continue;
        }

        colour[start_index] = GREY;
        // Each frame is a node index plus how many children we have walked.
        let mut stack: Vec<(usize, usize)> = vec![(start_index, 0)];
        while let Some(&(node_index, cursor)) = stack.last() {
            let node_children = children[node_index];
            match (cursor < node_children.len()).then(|| node_children[cursor]) {
                Some(child_id) => {
                    stack.last_mut().unwrap().1 += 1;

                    let Some(&child_index) = index_of.get(&child_id) else {
                        continue;
                    };

                    match colour[child_index] {
                        WHITE => {
                            colour[child_index] = GREY;
                            stack.push((child_index, 0));
                        }
                        GREY => return Some(child_index),
                        _ => {}
                    }
                }
                None => {
                    colour[node_index] = BLACK;
                    stack.pop();
                }
            }
        }
    }

    None
}
