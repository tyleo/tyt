use crate::{IDENTITY_AXIS_ANGLE, VMaxExt, VMaxExtNode, synth_uuid};
use std::collections::HashSet;
use ty_math::TyQuaternionF64;
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
        rotation: axis_angle(node.transform.rotation),
        alignment: DEFAULT_ALIGNMENT.to_owned(),
        pivot_face: DEFAULT_PIVOT_FACE.to_owned(),
        pivot_align: DEFAULT_PIVOT_ALIGN.to_owned(),
        selected: None,
    }
}

/// The `[x, y, z, angle]` axis-angle that reproduces a quaternion rotation,
/// the inverse of the writer's decode. Feeding the result back through the
/// decode, and Voxel Max's, recovers the same rotation.
pub fn axis_angle(rotation: TyQuaternionF64) -> [f64; 4] {
    let (axis, angle) = rotation.to_axis_angle();
    if angle == 0.0 {
        // No rotation: match Voxel Max's `[0, 0, 0, 0]` rather than emit a bare
        // axis.
        return IDENTITY_AXIS_ANGLE;
    }
    [axis.x, axis.y, axis.z, angle]
}
