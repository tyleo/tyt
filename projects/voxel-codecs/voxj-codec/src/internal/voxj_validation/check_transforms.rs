use crate::{Check, Failures};
use voxj::VoxjMain;

/// How far a rotation quaternion's length-squared may stray from `1` and still
/// count as a unit quaternion.
const ROTATION_TOLERANCE: f64 = 1e-6;

/// No transform scale component is zero and every rotation is a unit
/// quaternion.
pub fn check_transforms(main: &VoxjMain, failures: &mut Failures) {
    for (index, node) in main.runtime_state.nodes.iter().enumerate() {
        if !failures.go() {
            return;
        }
        let [scale_x, scale_y, scale_z] = node.transform.scale;
        if scale_x == 0.0 || scale_y == 0.0 || scale_z == 0.0 {
            failures.report(
                Check::Scale,
                format!("hierarchy node {index} has a transform scale component of zero"),
            );
            if !failures.go() {
                return;
            }
        }

        let [rotation_x, rotation_y, rotation_z, rotation_w] = node.transform.rotation;
        let length_squared = rotation_x * rotation_x
            + rotation_y * rotation_y
            + rotation_z * rotation_z
            + rotation_w * rotation_w;
        if (length_squared - 1.0).abs() > ROTATION_TOLERANCE {
            failures.report(
                Check::Rotation,
                format!(
                    "hierarchy node {index} rotation is not a unit quaternion \
                     (length squared {length_squared})"
                ),
            );
            if !failures.go() {
                return;
            }
        }
    }
}
