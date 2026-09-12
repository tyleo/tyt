use crate::{
    VMaxExtNode, VMaxExtObjectState, VMaxExtPalette, synthesized_node, synthesized_object_state,
};
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
    mem,
};
use vmax::VMaxSceneJsonFile;
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, Error as VoxError,
    Result as VoxResult, VoxExt, VoxGcRemap, VoxState,
    material::{BASE_COLOR, EMISSIVE_COLOR},
};

/// The `vmax` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Voxel Max state with no native voxcore home, kept so a document loaded from
/// a Voxel Max package can be written back exactly.
///
/// Every entry is keyed by its entity's id and every live entity has one. An
/// entity retained after the load takes a synthesized entry as it is
/// retained, and a released one loses its entry. The ext holds only what the
/// scene cannot derive. A node's name, position, scale, and parent, and an
/// object's content box, are read from the scene on write.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxExt {
    /// The Voxel Max scene with its hierarchy emptied, holding only the
    /// scene-level state recorded around the objects and groups that voxcore
    /// represents natively.
    pub scene: VMaxSceneJsonFile,

    /// Per-node provenance by hierarchy node id.
    #[cfg_attr(feature = "serde", serde(rename = "hierarchy-nodes"))]
    pub hierarchy_nodes: BTreeMap<U32Id<BVoxHierarchyNode>, VMaxExtNode>,

    /// Per-palette provenance by palette id.
    pub palettes: BTreeMap<U32Id<BVoxPalette>, VMaxExtPalette>,

    /// Per-object editor state by object id.
    #[cfg_attr(feature = "serde", serde(rename = "object-states"))]
    pub object_states: BTreeMap<U32Id<BVoxObject>, VMaxExtObjectState>,
}

fn refuse(reason: impl Display) -> VoxError {
    VoxError::Ext {
        reason: reason.to_string(),
    }
}

fn insert<K: Copy + Display + Ord, V>(
    map: &mut BTreeMap<K, V>,
    id: K,
    entry: V,
    what: &str,
) -> VoxResult<()> {
    if map.insert(id, entry).is_some() {
        return Err(refuse(format!(
            "vmax ext already holds an entry for {what} {id}"
        )));
    }
    Ok(())
}

fn remove<K: Copy + Display + Ord, V>(map: &mut BTreeMap<K, V>, id: K, what: &str) -> VoxResult<V> {
    map.remove(&id)
        .ok_or_else(|| refuse(format!("vmax ext holds no entry for {what} {id}")))
}

fn rekey<K: Copy + Display + Ord, V>(
    map: BTreeMap<K, V>,
    new_id: impl Fn(K) -> Option<K>,
    what: &str,
) -> VoxResult<BTreeMap<K, V>> {
    map.into_iter()
        .map(|(id, entry)| {
            let Some(id) = new_id(id) else {
                return Err(refuse(format!(
                    "gc dropped {what} {id}, which the vmax ext holds"
                )));
            };
            Ok((id, entry))
        })
        .collect()
}

/// A retained node, object, or palette takes the entry
/// [`to_vmax_vox_main`](crate::to_vmax_vox_main) would synthesize for it. A
/// retained material in a palette with an exact material list takes the slot
/// its material-axis value ids select.
impl VoxExt for VMaxExt {
    fn hierarchy_node_did_retain(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        let node = state
            .hierarchy_node(node_id)
            .expect("a retained node is live");
        let entry = synthesized_node(self, node);
        insert(&mut self.hierarchy_nodes, node_id, entry, "node")
    }

    fn hierarchy_node_will_release(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        remove(&mut self.hierarchy_nodes, node_id, "node").map(|_| ())
    }

    fn object_did_retain(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> VoxResult<()> {
        let object = state.object(object_id).expect("a retained object is live");
        let entry = synthesized_object_state(self, object);
        insert(&mut self.object_states, object_id, entry, "object")
    }

    fn object_will_release(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> VoxResult<()> {
        remove(&mut self.object_states, object_id, "object").map(|_| ())
    }

    fn palette_did_retain(
        &mut self,
        _state: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> VoxResult<()> {
        insert(
            &mut self.palettes,
            palette_id,
            VMaxExtPalette::default(),
            "palette",
        )
    }

    fn palette_will_release(
        &mut self,
        _state: &VoxState,
        palette_id: U32Id<BVoxPalette>,
    ) -> VoxResult<()> {
        remove(&mut self.palettes, palette_id, "palette").map(|_| ())
    }

    fn material_did_retain(
        &mut self,
        state: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_id: U32Id<BVoxMaterial>,
    ) -> VoxResult<()> {
        let Some(provenance) = self.palettes.get_mut(&palette_id) else {
            return Err(refuse(format!(
                "vmax ext holds no entry for palette {}",
                palette_id.to_u32()
            )));
        };
        if provenance.materials.is_empty() {
            return Ok(());
        }
        let slot = material_slot(state, palette_id, material_id, provenance.materials.len())?;
        insert(&mut provenance.slots, material_id, slot, "material")
    }

    fn materials_will_release(
        &mut self,
        _state: &VoxState,
        palette_id: U32Id<BVoxPalette>,
        material_ids: &[U32Id<BVoxMaterial>],
    ) -> VoxResult<()> {
        let Some(provenance) = self.palettes.get_mut(&palette_id) else {
            return Err(refuse(format!(
                "vmax ext holds no entry for palette {}",
                palette_id.to_u32()
            )));
        };
        if provenance.materials.is_empty() {
            return Ok(());
        }
        for &material_id in material_ids {
            remove(&mut provenance.slots, material_id, "material")?;
        }
        Ok(())
    }

    fn did_gc(&mut self, _state: &VoxState, remap: &VoxGcRemap) -> VoxResult<()> {
        let hierarchy_nodes = mem::take(&mut self.hierarchy_nodes);
        self.hierarchy_nodes = rekey(
            hierarchy_nodes,
            |id| remap.hierarchy_nodes.new_id(id),
            "node",
        )?;

        let object_states = mem::take(&mut self.object_states);
        self.object_states = rekey(object_states, |id| remap.objects.new_id(id), "object")?;

        let palettes = mem::take(&mut self.palettes);
        for (old_id, mut provenance) in palettes {
            let Some(new_id) = remap.palettes.new_id(old_id) else {
                return Err(refuse(format!(
                    "gc dropped palette {}, which the vmax ext holds",
                    old_id.to_u32()
                )));
            };
            let slots = mem::take(&mut provenance.slots);
            provenance.slots = rekey(
                slots,
                |id| remap.materials[old_id.to_usize_id()].new_id(id),
                "material",
            )?;
            self.palettes.insert(new_id, provenance);
        }
        Ok(())
    }
}

/// The Voxel Max material slot a folded material draws: the value id every
/// material-axis property holds for it, which the loader set from the material
/// byte. Refuses when the properties disagree or when a value pool no longer
/// lists one value per exact material, because a pruned pool no longer indexes
/// the list.
fn material_slot(
    state: &VoxState,
    palette_id: U32Id<BVoxPalette>,
    material_id: U32Id<BVoxMaterial>,
    slot_count: usize,
) -> VoxResult<u8> {
    let palette = state.palette(palette_id).expect("a live palette");
    let mut slots: HashSet<u32> = HashSet::new();
    for (property_id, property) in palette.iter_properties() {
        if property.name == BASE_COLOR || property.name == EMISSIVE_COLOR {
            continue;
        }
        let value_pool = state
            .value_pool(property.value_pool_id)
            .expect("a property names a live value pool");
        if value_pool.len() != slot_count {
            return Err(refuse(format!(
                "`{}` lists {} values but the vmax ext lists {slot_count} exact materials, so \
                 the value pools no longer index the material list",
                property.name,
                value_pool.len()
            )));
        }
        let value_id = palette
            .value_id(material_id, property_id)
            .expect("a live material has a value id for every property");
        slots.insert(value_id.to_u32());
    }
    let mut slots: Vec<u32> = slots.into_iter().collect();
    slots.sort_unstable();
    match slots.as_slice() {
        [slot] => Ok(*slot as u8),
        [] => Err(refuse(
            "the palette has no material property to read a Voxel Max material slot from",
        )),
        _ => Err(refuse(format!(
            "material {} draws different material-axis values {slots:?}, so it selects no \
             single Voxel Max material slot",
            material_id.to_u32()
        ))),
    }
}
