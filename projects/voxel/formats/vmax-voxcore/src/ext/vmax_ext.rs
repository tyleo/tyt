use crate::{
    VMaxExtNode, VMaxExtObjectState, VMaxExtPalette, mesh_texture_file, place_object,
    synthesized_node, synthesized_object_state,
};
use branded_id::{IdRange, U32Id};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::Display,
    mem,
};
use ty_math::TyVector3U32;
use vmax::VMaxSceneJsonFile;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxVoxel, Error as VoxError, Result as VoxResult,
    VoxExt, VoxGcRemap, VoxObject, VoxState,
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

    /// Every package file the scene does not model, by package path:
    /// external meshes and their textures, animations, and any file a later
    /// Voxel Max adds. Each entry holds the standard base64 of the file's
    /// bytes, which the write decodes and puts back.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "other-files",
            default,
            skip_serializing_if = "BTreeMap::is_empty"
        )
    )]
    pub other_files: BTreeMap<String, String>,
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

fn release_file(other_files: &mut BTreeMap<String, String>, path: &str) -> VoxResult<()> {
    other_files.remove(path).map(|_| ()).ok_or_else(|| {
        refuse(format!(
            "vmax ext holds no file `{path}` for a released external mesh"
        ))
    })
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

/// An external mesh's files go with the last node placing it. A retained node,
/// object, or palette takes the entry
/// [`to_vmax_vox_main`](crate::to_vmax_vox_main()) would synthesize for it. A
/// voxel remap or resample moves an object's kept camera target by as much as
/// its content center moved, which keeps the author's framing. A gc rekeys every
/// entry. A palette's slots follow its material pools' surviving values.
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
        let entry = remove(&mut self.hierarchy_nodes, node_id, "node")?;
        let Some(mesh) = entry.external_mesh else {
            return Ok(());
        };
        self.release_mesh_files(&mesh.data)
    }

    fn object_did_retain(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> VoxResult<()> {
        let entry = synthesized_object_state(self);
        insert(&mut self.object_states, object_id, entry, "object")
    }

    fn object_will_release(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> VoxResult<()> {
        remove(&mut self.object_states, object_id, "object").map(|_| ())
    }

    fn object_voxels_did_remap(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        voxel_ids: &HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>,
    ) -> VoxResult<()> {
        self.follow_content_center(state, object_id, old_bounds, voxel_ids.keys().copied())
    }

    fn object_voxels_did_resample(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        old_voxel_ids: &[U32Id<BVoxVoxel>],
    ) -> VoxResult<()> {
        self.follow_content_center(state, object_id, old_bounds, old_voxel_ids.iter().copied())
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
        for (old_id, provenance) in palettes {
            let Some(new_id) = remap.palettes.new_id(old_id) else {
                return Err(refuse(format!(
                    "gc dropped palette {old_id}, which the vmax ext holds"
                )));
            };
            self.palettes
                .insert(new_id, compacted_slots(state, remap, new_id, provenance));
        }
        Ok(())
    }
}

impl VMaxExt {
    /// Drops the files of the external mesh `data` once no node places it.
    fn release_mesh_files(&mut self, data: &str) -> VoxResult<()> {
        let placed = self
            .hierarchy_nodes
            .values()
            .filter_map(|entry| entry.external_mesh.as_ref())
            .any(|mesh| mesh.data == data);
        if placed {
            return Ok(());
        }
        release_file(&mut self.other_files, data)?;

        let Some(trsc) = &mut self.scene.trsc else {
            return Ok(());
        };
        let Some(uuids) = trsc.remove(data) else {
            return Ok(());
        };
        for uuid in uuids.into_iter().collect::<BTreeSet<_>>() {
            if !trsc.values().flatten().any(|other| *other == uuid) {
                release_file(&mut self.other_files, &mesh_texture_file(&uuid))?;
            }
        }
        Ok(())
    }

    /// Moves the camera target of object `object_id` by as much as its content
    /// center moved off a grid of `old_bounds` live at `old_voxel_ids`.
    fn follow_content_center(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        old_voxel_ids: impl Iterator<Item = U32Id<BVoxVoxel>>,
    ) -> VoxResult<()> {
        let Some(object_state) = self.object_states.get_mut(&object_id) else {
            return Err(refuse(format!(
                "vmax ext holds no entry for object {object_id}"
            )));
        };
        let Some(cam) = object_state.cam.as_mut() else {
            return Ok(());
        };

        let object = state.object(object_id).expect("a regridded object is live");
        let (_, placement) = place_object(object);
        let old_center = content_center(old_bounds, old_voxel_ids);
        for (axis, ((target, center), old_center)) in cam
            .o
            .iter_mut()
            .zip(placement.center)
            .zip(old_center)
            .enumerate()
        {
            let delta = center - old_center;
            *target += delta;
            if let Some(reference) = &mut object_state.camera_reference_center {
                reference[axis] += delta;
            }
        }
        Ok(())
    }
}

/// The content center (`e_c`) of an object on a grid of `bounds` live at
/// `voxel_ids`.
fn content_center(
    bounds: TyVector3U32,
    voxel_ids: impl Iterator<Item = U32Id<BVoxVoxel>>,
) -> [f64; 3] {
    let mut shell =
        VoxObject::new(String::new(), bounds).expect("the old grid was within the cell cap");
    for voxel_id in voxel_ids {
        shell
            .retain_voxel(voxel_id, &[])
            .expect("an old live voxel is within the old grid");
    }
    place_object(&shell).1.center
}

/// Palette `palette_id`'s provenance after a gc of the material pools its
/// slots index. A slot the gc dropped loses its entries. A palette binding no
/// material axis keeps its slots as they are.
fn compacted_slots(
    state: &VoxState,
    remap: &VoxGcRemap,
    palette_id: U32Id<BVoxPalette>,
    provenance: VMaxExtPalette,
) -> VMaxExtPalette {
    let palette = state.palette(palette_id).expect("a live palette");
    let Some(new_pool_id) = palette
        .iter_properties()
        .map(|(_, property)| property)
        .find(|property| property.name != BASE_COLOR && property.name != EMISSIVE_COLOR)
        .map(|property| property.value_pool_id)
    else {
        return provenance;
    };
    // The value remap is keyed by the pool's old id.
    let old_pool_id = IdRange::<U32Id<_>>::from_len(remap.value_pools.old_len())
        .find(|&old_id| remap.value_pools.new_id(old_id) == Some(new_pool_id))
        .expect("a live pool has an old id");
    let values = &remap.value_pool_values[old_pool_id.to_usize_id()];

    VMaxExtPalette {
        name: provenance.name,
        slots_without_dispersion: provenance
            .slots_without_dispersion
            .into_iter()
            .filter_map(|slot| values.new_id(slot))
            .collect(),
        transmission_colors: provenance
            .transmission_colors
            .into_iter()
            .filter_map(|(slot, color)| Some((values.new_id(slot)?, color)))
            .collect(),
    }
}
