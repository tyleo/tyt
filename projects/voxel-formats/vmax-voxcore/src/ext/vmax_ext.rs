use crate::{
    VMaxExtMaterial, VMaxExtNode, VMaxExtObjectState, VMaxExtPalette, synthesized_node,
    synthesized_object_state,
};
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Display, mem};
use vmax::VMaxSceneJsonFile;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, Error as VoxError, Result as VoxResult, VoxExt,
    VoxGcRemap, VoxState,
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
/// gc rekeys every entry. An exact material list follows its material pools'
/// surviving values.
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

    fn did_gc(&mut self, state: &VoxState, remap: &VoxGcRemap) -> VoxResult<()> {
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
            if !provenance.materials.is_empty() {
                let materials = mem::take(&mut provenance.materials);
                provenance.materials = compacted_materials(state, remap, new_id, materials)?;
            }
            self.palettes.insert(new_id, provenance);
        }
        Ok(())
    }
}

/// An exact material list after a gc of the material pools it indexes by
/// slot: the surviving values' materials, in their compacted order. A palette
/// binding no material axis keeps its list as it is. Refuses when a compacted
/// value indexes no material.
fn compacted_materials(
    state: &VoxState,
    remap: &VoxGcRemap,
    palette_id: U32Id<BVoxPalette>,
    materials: Vec<VMaxExtMaterial>,
) -> VoxResult<Vec<VMaxExtMaterial>> {
    let palette = state.palette(palette_id).expect("a live palette");
    let Some(new_pool_id) = palette
        .iter_properties()
        .map(|(_, property)| property)
        .find(|property| property.name != BASE_COLOR && property.name != EMISSIVE_COLOR)
        .map(|property| property.value_pool_id)
    else {
        return Ok(materials);
    };
    // The value remap is keyed by the pool's old id.
    let old_pool_id = (0..remap.value_pools.old_len())
        .map(|old| U32Id::from_u32(old as u32))
        .find(|&old| remap.value_pools.new_id(old) == Some(new_pool_id))
        .expect("a live pool has an old id");
    let values = &remap.value_pool_values[old_pool_id.to_usize_id()];
    let mut compacted: Vec<Option<VMaxExtMaterial>> = (0..values.new_len()).map(|_| None).collect();
    for (slot, material) in materials.into_iter().enumerate() {
        if let Some(new_id) = values.new_id(U32Id::from_u32(slot as u32)) {
            compacted[new_id.to_u32() as usize] = Some(material);
        }
    }
    compacted
        .into_iter()
        .enumerate()
        .map(|(slot, material)| {
            material.ok_or_else(|| {
                refuse(format!(
                    "material pool value {slot} indexes no exact material in the vmax ext"
                ))
            })
        })
        .collect()
}
