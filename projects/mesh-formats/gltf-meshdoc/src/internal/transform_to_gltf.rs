use crate::yup_to_zup_rotation;
use ty_math::{TyTransformF64, TyVector3Ext};

/// A node's transform on glTF's Y-up axes as its translation, rotation, and
/// scale, the inverse of the read's frame change.
pub fn transform_to_gltf(transform: &TyTransformF64) -> ([f32; 3], [f32; 4], [f32; 3]) {
    let frame = yup_to_zup_rotation();

    let rotation = (frame.inverse() * transform.rotation * frame).normalize();

    (
        transform
            .position
            .zup_to_yup()
            .to_array()
            .map(|value| value as f32),
        [
            rotation.x as f32,
            rotation.y as f32,
            rotation.z as f32,
            rotation.w as f32,
        ],
        [
            transform.scale.x as f32,
            transform.scale.z as f32,
            transform.scale.y as f32,
        ],
    )
}
