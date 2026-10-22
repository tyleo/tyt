use branded_id::{IdVec, U32Id};
use sdfcore::{
    SdfMain, SdfMaterial, SdfNode, SdfObject, SdfShape3d, SdfState, SdfStep, SdfStepMaterial,
};
use ty_math::TyVector3F64;

/// A model whose root part `model` holds the parts `a` and `b`, which both
/// place the part `leaf` and its unit box. Every part turns about
/// `[0.5, 0, 0]`, `a` moves by `[2, 0, 0]`, and `b` moves by `[0, 0.25, 0]`.
pub fn shared_leaf_main() -> SdfMain {
    let node = |name: &str, offset: Option<[f64; 3]>, objects: Vec<u32>, nodes: Vec<u32>| SdfNode {
        name: name.to_string(),
        pivot: Some(TyVector3F64::new(0.5, 0.0, 0.0)),
        offset: offset.map(TyVector3F64::from_array),
        rotation: Vec::new(),
        joint: false,
        child_object_ids: objects.into_iter().map(U32Id::from_u32).collect(),
        child_node_ids: nodes.into_iter().map(U32Id::from_u32).collect(),
    };

    SdfMain::new(SdfState {
        shapes3d: IdVec::from_vec(vec![SdfShape3d::Box {
            min: TyVector3F64::ZERO,
            max: TyVector3F64::ONE,
            round: None,
        }]),
        materials: IdVec::from_vec(vec![SdfMaterial::Material {
            properties: Vec::new(),
        }]),
        steps: IdVec::from_vec(vec![SdfStep::Add {
            name: "cube".to_string(),
            shape_id: U32Id::from_u32(0),
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
        }]),
        objects: IdVec::from_vec(vec![SdfObject {
            name: "leaf".to_string(),
            step_ids: vec![U32Id::from_u32(0)],
        }]),
        nodes: IdVec::from_vec(vec![
            node("leaf", None, vec![0], Vec::new()),
            node("a", Some([2.0, 0.0, 0.0]), Vec::new(), vec![0]),
            node("b", Some([0.0, 0.25, 0.0]), Vec::new(), vec![0]),
            node("model", None, Vec::new(), vec![1, 2]),
        ]),
        root_node_ids: vec![U32Id::from_u32(3)],
        ..SdfState::default()
    })
    .unwrap()
}
