use crate::{IDENTITY_AXIS_ANGLE, VMaxExtNode};
use ty_math::TyQuaternionF64;
use voxcore::VoxHierarchyNode;

/// Default transform-anchor tokens for a synthesized node. Voxel Max decodes
/// each as an enum and rejects an empty token.
const DEFAULT_ALIGNMENT: &str = "f";

const DEFAULT_PIVOT_ALIGN: &str = "4";

const DEFAULT_PIVOT_FACE: &str = "8";

/// The provenance of a node the document never carried. The content box and
/// placement are derived from the native bounds and node transform on write,
/// so it holds the ids, the node's rotation encoded from its live quaternion,
/// and the default anchor tokens.
pub fn synthesized_node(
    id: String,
    parent_id: Option<String>,
    node: &VoxHierarchyNode,
) -> VMaxExtNode {
    VMaxExtNode {
        id,
        parent_id,
        index: None,
        rotation: Some(axis_angle(node.transform.rotation)),
        alignment: Some(DEFAULT_ALIGNMENT.to_owned()),
        pivot_face: Some(DEFAULT_PIVOT_FACE.to_owned()),
        pivot_align: Some(DEFAULT_PIVOT_ALIGN.to_owned()),
        selected: None,
    }
}

/// The `[x, y, z, angle]` axis-angle that reproduces a quaternion rotation,
/// the inverse of the writer's decode. A synthesized node has no preserved
/// `t_r`, so its rotation is encoded from the live quaternion. Feeding the
/// result back through the decode, and Voxel Max's, recovers the same
/// rotation.
fn axis_angle(rotation: TyQuaternionF64) -> [f64; 4] {
    let (axis, angle) = rotation.to_axis_angle();
    if angle == 0.0 {
        // No rotation: match Voxel Max's `[0, 0, 0, 0]` rather than emit a bare
        // axis.
        return IDENTITY_AXIS_ANGLE;
    }
    [axis.x, axis.y, axis.z, angle]
}
