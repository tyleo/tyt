use gltf::Node;
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};

/// A node's transform: its TRS, or its matrix decomposed.
pub fn transform_from_gltf(node: &Node) -> TyTransformF64 {
    let (translation, rotation, scale) = node.transform().decomposed();

    TyTransformF64 {
        position: TyVector3F64::from_array(translation.map(f64::from)),
        rotation: TyQuaternionF64::from_xyzw(
            f64::from(rotation[0]),
            f64::from(rotation[1]),
            f64::from(rotation[2]),
            f64::from(rotation[3]),
        ),
        scale: TyVector3F64::from_array(scale.map(f64::from)),
    }
}
