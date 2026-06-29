use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyQuaternion, TyTransformF64, TyVector3};
use voxcore::VoxHierarchyNode;
use voxj::{VoxjHierarchyNode, VoxjTransform};

/// Builds a [`VoxHierarchyNode`] from a [`VoxjHierarchyNode`], mapping child
/// indices to ids and the transform to its [`ty_math`] form. Child ids are
/// checked by [`VoxMain::validate`](voxcore::VoxMain::validate), not here.
///
/// Errors on a degenerate transform: non-finite position, non-finite or zero
/// scale, or a non-finite / zero rotation.
pub fn vox_hierarchy_node_from_voxj_hierarchy_node(
    node: &VoxjHierarchyNode,
) -> Result<VoxHierarchyNode> {
    Ok(VoxHierarchyNode {
        name: node.name.clone(),
        child_nodes: node
            .child_nodes
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
        child_objects: node
            .child_objects
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
        transform: vox_transform_from_voxj_transform(&node.transform)?,
    })
}

/// Converts a [`VoxjTransform`] into a [`TyTransformF64`], validating it:
/// position finite, scale finite and non-zero, rotation finite and non-zero.
/// The rotation is normalized (tolerating a unit quaternion's float error).
fn vox_transform_from_voxj_transform(transform: &VoxjTransform) -> Result<TyTransformF64> {
    let [position_x, position_y, position_z] = transform.position;
    let [rotation_x, rotation_y, rotation_z, rotation_w] = transform.rotation;
    let [scale_x, scale_y, scale_z] = transform.scale;

    for value in [position_x, position_y, position_z] {
        if !value.is_finite() {
            return Err(invalid(format!(
                "transform position component {value} must be finite"
            )));
        }
    }

    for value in [scale_x, scale_y, scale_z] {
        if !value.is_finite() || value == 0.0 {
            return Err(invalid(format!(
                "transform scale component {value} must be finite and non-zero"
            )));
        }
    }

    for value in [rotation_x, rotation_y, rotation_z, rotation_w] {
        if !value.is_finite() {
            return Err(invalid(format!(
                "transform rotation component {value} must be finite"
            )));
        }
    }
    let magnitude = (rotation_x * rotation_x
        + rotation_y * rotation_y
        + rotation_z * rotation_z
        + rotation_w * rotation_w)
        .sqrt();
    if magnitude == 0.0 {
        return Err(invalid(
            "transform rotation quaternion must not be zero".to_owned(),
        ));
    }

    Ok(TyTransformF64::new(
        TyVector3::new(position_x, position_y, position_z),
        TyQuaternion::new(
            rotation_x / magnitude,
            rotation_y / magnitude,
            rotation_z / magnitude,
            rotation_w / magnitude,
        ),
        TyVector3::new(scale_x, scale_y, scale_z),
    ))
}

/// Invalid-data error from a message.
fn invalid(message: String) -> Error {
    Error::Invalid(message)
}

#[cfg(test)]
mod tests {
    use crate::vox_hierarchy_node_from_voxj_hierarchy_node;
    use voxj::{VoxjHierarchyNode, VoxjTransform};

    fn node_with_transform(transform: VoxjTransform) -> VoxjHierarchyNode {
        VoxjHierarchyNode {
            name: "n".to_owned(),
            child_nodes: Vec::new(),
            child_objects: Vec::new(),
            transform,
        }
    }

    #[test]
    fn rejects_zero_scale_and_non_finite_components() {
        let zero_scale = node_with_transform(VoxjTransform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 0.0, 1.0],
        });
        assert!(vox_hierarchy_node_from_voxj_hierarchy_node(&zero_scale).is_err());

        let nan_position = node_with_transform(VoxjTransform {
            position: [f64::NAN, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        });
        assert!(vox_hierarchy_node_from_voxj_hierarchy_node(&nan_position).is_err());
    }

    #[test]
    fn normalizes_a_non_unit_rotation() {
        let node = node_with_transform(VoxjTransform {
            position: [0.0, 0.0, 0.0],
            // Magnitude 2; should normalize to the unit identity (w = 1).
            rotation: [0.0, 0.0, 0.0, 2.0],
            scale: [1.0, 1.0, 1.0],
        });
        let rotation = vox_hierarchy_node_from_voxj_hierarchy_node(&node)
            .unwrap()
            .transform
            .rotation;
        assert_eq!(
            (rotation.x, rotation.y, rotation.z, rotation.w),
            (0.0, 0.0, 0.0, 1.0)
        );
    }
}
