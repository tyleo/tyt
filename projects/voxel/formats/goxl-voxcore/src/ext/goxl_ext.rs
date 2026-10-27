use crate::{
    GoxlExtCamera, GoxlExtImage, GoxlExtLayer, GoxlExtLight, GoxlExtMaterial, GoxlExtPlacement,
    GoxlExtPreview, GoxlExtUnknownChunk, next_layer_id, synthesized_layer,
};
use branded_id::{SliceExt, U32Id};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use ty_math::{TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxVoxel, Error as VoxError, Result as VoxResult, VoxExt,
    VoxGcRemap, VoxObject, VoxState,
};

/// The `goxl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Goxel `.gox` state with no native voxcore home, kept so a file loaded from a
/// Goxel package can be written back exactly.
///
/// The shared `BL16` voxel blocks become native objects and the per-layer block
/// placements that stamp them become the hierarchy nodes; this holds the rest,
/// with one layer entry per hierarchy node, keyed by the node's id. The entries
/// follow the state through the [`VoxExt`] hooks.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct GoxlExt {
    /// The format version from the header.
    pub version: i32,

    /// The `IMG ` image metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub image: GoxlExtImage,

    /// The `PREV` preview thumbnail, or `None` when the file omits it.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub preview: Option<GoxlExtPreview>,

    /// The `MATE` materials, in stored order; a layer names one by index.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub materials: Vec<GoxlExtMaterial>,

    /// Per-layer provenance, one entry per hierarchy node, keyed by node id.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "BTreeMap::is_empty")
    )]
    pub layers: BTreeMap<U32Id<BVoxHierarchyNode>, GoxlExtLayer>,

    /// The `CAMR` cameras, in stored order.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub cameras: Vec<GoxlExtCamera>,

    /// The `LIGH` light settings, or `None` when the file omits them.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub light: Option<GoxlExtLight>,

    /// Chunks the goxl crate does not model, preserved verbatim.
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "unknown-chunks",
            default,
            skip_serializing_if = "Vec::is_empty"
        )
    )]
    pub unknown_chunks: Vec<GoxlExtUnknownChunk>,
}

/// Where a box of `bounds` at `origin` lands on Goxel's Z-up axes under a
/// node at `node_position`, as the writer turns the object.
fn stamp_position(
    node_position: TyVector3F64,
    origin: TyVector3I32,
    bounds: TyVector3U32,
) -> TyVector3I32 {
    let corner = node_position.round().as_ivec3() + origin;
    TyVector3I32::new(corner.x, -(corner.z + bounds.z as i32), corner.y)
}

fn object_stamp_position(node_position: TyVector3F64, object: &VoxObject) -> [i32; 3] {
    stamp_position(node_position, object.origin(), object.bounds()).to_array()
}

fn layer_mut(
    layers: &mut BTreeMap<U32Id<BVoxHierarchyNode>, GoxlExtLayer>,
    node_id: U32Id<BVoxHierarchyNode>,
) -> VoxResult<&mut GoxlExtLayer> {
    layers.get_mut(&node_id).ok_or_else(|| VoxError::Ext {
        reason: format!("goxl ext has no layer entry for node {node_id}"),
    })
}

fn shift_object_placements(
    layers: &mut BTreeMap<U32Id<BVoxHierarchyNode>, GoxlExtLayer>,
    state: &VoxState,
    object_id: U32Id<BVoxObject>,
    shift: impl Fn(TyVector3F64) -> TyVector3I32,
) {
    for (&node_id, layer) in layers.iter_mut() {
        let node = state
            .hierarchy_node(node_id)
            .expect("a layer's node is live");
        let delta = shift(node.transform.position);
        for placement in &mut layer.placements {
            if placement.object_id == object_id {
                placement.position =
                    (TyVector3I32::from_array(placement.position) + delta).to_array();
            }
        }
    }
}

impl GoxlExt {
    /// Moves each placement of object `object_id` by as much as its stamped
    /// box moved off a grid of `old_bounds`.
    fn follow_bounds(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
    ) {
        let object = state.object(object_id).expect("a regridded object is live");
        shift_object_placements(&mut self.layers, state, object_id, |node_position| {
            stamp_position(node_position, object.origin(), object.bounds())
                - stamp_position(node_position, object.origin(), old_bounds)
        });
    }
}

/// A node another entry clones refuses its release, because the clone's
/// `base_id` would dangle. A node transform, object origin, or object bounds
/// change shifts each affected placement by as much as the stamped box moved,
/// which keeps the authored offsets.
impl VoxExt for GoxlExt {
    fn hierarchy_node_did_retain(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        let node = state
            .hierarchy_node(node_id)
            .expect("a retained node is live");
        let placements = node
            .child_object_ids
            .iter()
            .map(|&object_id| GoxlExtPlacement {
                object_id,
                position: object_stamp_position(
                    node.transform.position,
                    state.object(object_id).expect("a placed object is live"),
                ),
            })
            .collect();
        let layer = synthesized_layer(next_layer_id(&self.layers), placements);
        self.layers.insert(node_id, layer);
        Ok(())
    }

    fn hierarchy_node_will_release(
        &mut self,
        _state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        let Some(layer) = self.layers.get(&node_id) else {
            return Err(VoxError::Ext {
                reason: format!("goxl ext has no layer entry for node {node_id}"),
            });
        };

        let clone_node_ids: Vec<U32Id<BVoxHierarchyNode>> = self
            .layers
            .iter()
            .filter(|&(&other_id, other)| {
                other_id != node_id && layer.id != 0 && other.base_id == layer.id
            })
            .map(|(&other_id, _)| other_id)
            .collect();
        if !clone_node_ids.is_empty() {
            return Err(VoxError::Ext {
                reason: format!(
                    "node {node_id} is goxl layer id {}, which the layers of nodes {} clone; \
                     release the clones first",
                    layer.id,
                    clone_node_ids.display_ids()
                ),
            });
        }

        self.layers.remove(&node_id);
        Ok(())
    }

    fn hierarchy_node_transform_did_set(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        old_transform: TyTransformF64,
    ) -> VoxResult<()> {
        let node = state.hierarchy_node(node_id).expect("a set node is live");
        let layer = layer_mut(&mut self.layers, node_id)?;
        for placement in &mut layer.placements {
            let object = state
                .object(placement.object_id)
                .expect("a placed object is live");
            let delta = stamp_position(node.transform.position, object.origin(), object.bounds())
                - stamp_position(old_transform.position, object.origin(), object.bounds());
            placement.position = (TyVector3I32::from_array(placement.position) + delta).to_array();
        }
        Ok(())
    }

    fn hierarchy_node_children_did_set(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
        _old_child_node_ids: &[U32Id<BVoxHierarchyNode>],
        _old_child_object_ids: &[U32Id<BVoxObject>],
    ) -> VoxResult<()> {
        let node = state.hierarchy_node(node_id).expect("a set node is live");
        let layer = layer_mut(&mut self.layers, node_id)?;
        layer
            .placements
            .retain(|placement| node.child_object_ids.contains(&placement.object_id));
        for &object_id in &node.child_object_ids {
            if !layer
                .placements
                .iter()
                .any(|placement| placement.object_id == object_id)
            {
                layer.placements.push(GoxlExtPlacement {
                    object_id,
                    position: object_stamp_position(
                        node.transform.position,
                        state.object(object_id).expect("a placed object is live"),
                    ),
                });
            }
        }

        // The writer checks that the first-stamped order matches the child
        // order. When the new child order breaks it, the placements regroup
        // by child.
        let mut seen = HashSet::new();
        let stamped: Vec<_> = layer
            .placements
            .iter()
            .map(|placement| placement.object_id)
            .filter(|object_id| seen.insert(*object_id))
            .collect();
        if stamped != node.child_object_ids {
            layer.placements.sort_by_key(|placement| {
                node.child_object_ids
                    .iter()
                    .position(|&object_id| object_id == placement.object_id)
            });
        }
        Ok(())
    }

    fn object_origin_did_set(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_origin: TyVector3I32,
    ) -> VoxResult<()> {
        let object = state.object(object_id).expect("a set object is live");
        shift_object_placements(&mut self.layers, state, object_id, |node_position| {
            stamp_position(node_position, object.origin(), object.bounds())
                - stamp_position(node_position, old_origin, object.bounds())
        });
        Ok(())
    }

    fn object_voxels_did_remap(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        _voxel_ids: &HashMap<U32Id<BVoxVoxel>, U32Id<BVoxVoxel>>,
    ) -> VoxResult<()> {
        self.follow_bounds(state, object_id, old_bounds);
        Ok(())
    }

    fn object_voxels_did_resample(
        &mut self,
        state: &VoxState,
        object_id: U32Id<BVoxObject>,
        old_bounds: TyVector3U32,
        _old_voxel_ids: &[U32Id<BVoxVoxel>],
    ) -> VoxResult<()> {
        self.follow_bounds(state, object_id, old_bounds);
        Ok(())
    }

    fn object_will_release(
        &mut self,
        _state: &VoxState,
        object_id: U32Id<BVoxObject>,
    ) -> VoxResult<()> {
        for layer in self.layers.values_mut() {
            layer
                .placements
                .retain(|placement| placement.object_id != object_id);
        }
        Ok(())
    }

    fn did_gc(&mut self, _state: &VoxState, remap: &VoxGcRemap) -> VoxResult<()> {
        let mut rekeyed = BTreeMap::new();
        for (&old_node_id, layer) in &self.layers {
            let Some(node_id) = remap.hierarchy_nodes.new_id(old_node_id) else {
                return Err(VoxError::Ext {
                    reason: format!(
                        "gc released node {old_node_id}, which still has a goxl ext layer entry"
                    ),
                });
            };

            let mut layer = layer.clone();
            for placement in &mut layer.placements {
                let Some(object_id) = remap.objects.new_id(placement.object_id) else {
                    return Err(VoxError::Ext {
                        reason: format!(
                            "gc released object {}, which the goxl ext layer for node {} still \
                             stamps",
                            placement.object_id, old_node_id
                        ),
                    });
                };
                placement.object_id = object_id;
            }
            rekeyed.insert(node_id, layer);
        }
        self.layers = rekeyed;
        Ok(())
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use crate::{GoxlExt, GoxlExtPlacement, synthesized_layer};
    use branded_id::U32Id;

    /// The layer map keys and the placement object ids ride as bare `u32`s
    /// and come back branded.
    #[test]
    fn round_trips_the_id_keys_through_json() {
        let mut ext = GoxlExt::default();

        ext.layers.insert(
            U32Id::from_u32(7),
            synthesized_layer(
                1,
                vec![GoxlExtPlacement {
                    object_id: U32Id::from_u32(3),
                    position: [16, 0, -16],
                }],
            ),
        );

        let json = serde_json::to_string(&ext).unwrap();

        assert!(json.contains(r#""layers":{"7":"#));

        assert!(json.contains(r#""object-id":3"#));

        assert_eq!(serde_json::from_str::<GoxlExt>(&json).unwrap(), ext);
    }
}
