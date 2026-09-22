use ty_math::TyTransformF64;

/// A node's transform as the `f32` triples glTF stores.
pub fn transform_to_gltf(transform: &TyTransformF64) -> ([f32; 3], [f32; 4], [f32; 3]) {
    (
        transform.position.to_array().map(|value| value as f32),
        [
            transform.rotation.x as f32,
            transform.rotation.y as f32,
            transform.rotation.z as f32,
            transform.rotation.w as f32,
        ],
        transform.scale.to_array().map(|value| value as f32),
    )
}
