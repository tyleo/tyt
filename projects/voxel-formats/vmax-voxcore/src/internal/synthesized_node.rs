use crate::{VMaxExt, VMaxExtNode, encode_axis_angle};
use std::collections::HashSet;
use voxcore::VoxHierarchyNode;

/// Default transform-anchor tokens for a synthesized node. Voxel Max decodes
/// each as an enum and rejects an empty token.
const DEFAULT_ALIGNMENT: &str = "f";

const DEFAULT_PIVOT_ALIGN: &str = "4";

const DEFAULT_PIVOT_FACE: &str = "8";

/// The provenance of a node the document never carried, built against the
/// entries `ext` already holds so its UUID and index triplet are fresh. The
/// synthesizer and the retain hook both build entries here.
pub fn synthesized_node(ext: &VMaxExt, node: &VoxHierarchyNode) -> VMaxExtNode {
    let ids: HashSet<&str> = ext
        .hierarchy_nodes
        .values()
        .map(|entry| entry.id.as_str())
        .collect();
    let id = (0..)
        .map(synth_uuid)
        .find(|id| !ids.contains(id.as_str()))
        .expect("a fresh index exists");

    // Voxel Max collapses nodes that share a triplet. Groups take the `1`
    // lane and objects the `0` lane.
    let lane = i64::from(node.child_object_ids.is_empty());
    let indices: HashSet<[i64; 3]> = ext
        .hierarchy_nodes
        .values()
        .map(|entry| entry.index)
        .collect();
    let index = (0..)
        .map(|counter| [0, lane, counter])
        .find(|index| !indices.contains(index))
        .expect("a fresh counter exists");

    VMaxExtNode {
        id,
        index,
        rotation: encode_axis_angle(node.transform.yup_to_zup().rotation),
        alignment: DEFAULT_ALIGNMENT.to_owned(),
        pivot_face: DEFAULT_PIVOT_FACE.to_owned(),
        pivot_align: DEFAULT_PIVOT_ALIGN.to_owned(),
        selected: None,
        hidden: None,
        external_mesh: None,
    }
}

/// A syntactically valid, deterministic UUID for a synthesized scene node.
/// Voxel Max decodes a node's `id` and `pid` as a UUID and rejects a non-UUID
/// token. The index is offset by one so the first node avoids the all-zero nil
/// UUID. The fourth group stays zero, where an extra object under a node
/// stamps its slot.
fn synth_uuid(index: usize) -> String {
    format!("00000000-0000-0000-0000-{:012X}", index + 1)
}
