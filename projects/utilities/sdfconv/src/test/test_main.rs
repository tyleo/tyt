use branded_id::{IdVec, U32Id};
use sdfcore::{
    SdfMain, SdfMaterial, SdfNode, SdfObject, SdfProperty, SdfPropertyValue, SdfShape3d, SdfState,
    SdfStep, SdfStepMaterial,
};
use ty_math::TyVector3F64;

/// A model of one sphere under one material, as one part.
pub fn test_main() -> SdfMain {
    SdfMain::new(SdfState {
        shapes3d: IdVec::from(vec![SdfShape3d::Sphere {
            center: TyVector3F64::new(0.0, 0.5, 0.0),
            radius: 0.25,
        }]),
        materials: IdVec::from(vec![SdfMaterial::Material {
            properties: vec![SdfProperty {
                name: "baseColor".to_owned(),
                value: SdfPropertyValue::Text("#C8B8A0".to_owned()),
            }],
        }]),
        steps: IdVec::from(vec![SdfStep::Add {
            name: "ball".to_owned(),
            shape_id: U32Id::from_u32(0),
            material: SdfStepMaterial::Material(U32Id::from_u32(0)),
        }]),
        objects: IdVec::from(vec![SdfObject {
            name: "ball".to_owned(),
            step_ids: vec![U32Id::from_u32(0)],
        }]),
        nodes: IdVec::from(vec![SdfNode {
            name: "ball".to_owned(),
            pivot: None,
            offset: None,
            rotation: Vec::new(),
            child_object_ids: vec![U32Id::from_u32(0)],
            child_node_ids: Vec::new(),
        }]),
        root_node_ids: vec![U32Id::from_u32(0)],
        ..SdfState::default()
    })
    .expect("the test model follows every rule")
}
