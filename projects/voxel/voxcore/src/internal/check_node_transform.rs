use crate::{BVoxHierarchyNode, Error, Result};
use branded_id::U32Id;
use ty_math::{TyQuaternionExt, TyTransformF64, UNIT_ROTATION_TOLERANCE};

/// Checks the transform of node `node_id`: finite and non-degenerate. The
/// rotation needs no finiteness guard of its own: a non-finite component fails
/// the unit-length check.
pub fn check_node_transform(
    node_id: U32Id<BVoxHierarchyNode>,
    transform: &TyTransformF64,
) -> Result<()> {
    if !transform.position.is_finite() || !transform.scale.is_finite() {
        return Err(Error::NonFiniteTransform { node_id });
    }

    let scale = transform.scale;
    if scale.x == 0.0 || scale.y == 0.0 || scale.z == 0.0 {
        return Err(Error::ZeroScale { node_id });
    }

    if !transform
        .rotation
        .is_normalized_within(UNIT_ROTATION_TOLERANCE)
    {
        return Err(Error::NonUnitRotation { node_id });
    }

    Ok(())
}
