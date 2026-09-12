use crate::{
    GoxlExtCamera, GoxlExtImage, GoxlExtLayer, GoxlExtLight, GoxlExtMaterial, GoxlExtPlacement,
    GoxlExtPreview, GoxlExtUnknownChunk, next_layer_id, synthesized_layer,
};
use branded_id::U32Id;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, Error as VoxError, Result as VoxResult, VoxExt, VoxGcRemap,
    VoxState,
};

/// The `goxl` ext payload stashed on a [`VoxMain`](voxcore::VoxMain): the
/// Goxel `.gox` state with no native voxcore home, kept so a file loaded from a
/// Goxel package can be written back exactly.
///
/// The shared `BL16` voxel blocks become native objects and the per-layer block
/// placements that stamp them become the hierarchy nodes; this holds the rest,
/// with one layer entry per hierarchy node, keyed by the node's id. The
/// entries follow the state through the [`VoxExt`](voxcore::VoxExt) hooks.
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

/// A node another entry clones refuses its release, because the clone's
/// `base_id` would dangle.
impl VoxExt for GoxlExt {
    fn hierarchy_node_did_retain(
        &mut self,
        state: &VoxState,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> VoxResult<()> {
        let node = state
            .hierarchy_node(node_id)
            .expect("a retained node is live");
        let position = node.transform.position.round().as_ivec3().to_array();
        let placements = node
            .child_object_ids
            .iter()
            .map(|&object_id| GoxlExtPlacement {
                object_id,
                position,
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
                reason: format!("goxl ext has no layer entry for node {}", node_id.to_u32()),
            });
        };

        let clone_node_ids: Vec<u32> = self
            .layers
            .iter()
            .filter(|&(&other_id, other)| {
                other_id != node_id && layer.id != 0 && other.base_id == layer.id
            })
            .map(|(other_id, _)| other_id.to_u32())
            .collect();
        if !clone_node_ids.is_empty() {
            return Err(VoxError::Ext {
                reason: format!(
                    "node {} is goxl layer id {}, which the layers of nodes {clone_node_ids:?} \
                     clone; release the clones first",
                    node_id.to_u32(),
                    layer.id
                ),
            });
        }

        self.layers.remove(&node_id);
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
                        "gc released node {}, which still has a goxl ext layer entry",
                        old_node_id.to_u32()
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
                            placement.object_id.to_u32(),
                            old_node_id.to_u32()
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
