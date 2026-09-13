use gltf::Node;
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3Ext, TyVector3F64};

/// A node's transform on meshdoc's Z-up axes: its TRS, or its matrix
/// decomposed, rotated from glTF's Y-up.
pub fn transform_from_gltf(node: &Node) -> TyTransformF64 {
    let (translation, rotation, scale) = node.transform().decomposed();

    let rotation = TyQuaternionF64::from_xyzw(
        f64::from(rotation[0]),
        f64::from(rotation[1]),
        f64::from(rotation[2]),
        f64::from(rotation[3]),
    );

    // A basis change: rotate the frame, then conjugate the rotation into the
    // new frame. The scale is per local axis, and the change swaps the y and
    // z axes.
    let frame = yup_to_zup_rotation();

    TyTransformF64 {
        position: TyVector3F64::from_array(translation.map(f64::from)).yup_to_zup(),
        rotation: (frame * rotation * frame.inverse()).normalize(),
        scale: TyVector3F64::new(
            f64::from(scale[0]),
            f64::from(scale[2]),
            f64::from(scale[1]),
        ),
    }
}

/// The rotation that carries glTF's Y-up frame onto meshdoc's Z-up frame,
/// sending glTF `+y` to `+z` and `+z` to `-y`.
pub fn yup_to_zup_rotation() -> TyQuaternionF64 {
    TyQuaternionF64::from_rotation_x(std::f64::consts::FRAC_PI_2)
}

#[cfg(test)]
mod tests {
    use crate::yup_to_zup_rotation;
    use ty_math::{TyVector3Ext, TyVector3F64};

    /// The frame rotation agrees with the axis swap on the basis vectors.
    #[test]
    fn the_frame_rotation_matches_the_axis_swap() {
        let frame = yup_to_zup_rotation();

        for axis in [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z] {
            let rotated = frame * axis;
            let swapped = axis.yup_to_zup();
            assert!((rotated - swapped).length() < 1e-12, "{axis:?}");
        }
    }
}
