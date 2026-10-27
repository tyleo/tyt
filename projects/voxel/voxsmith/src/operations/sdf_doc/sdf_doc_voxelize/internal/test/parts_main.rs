use branded_id::{IdVec, U32Id};
use sdfcore::{
    SdfMain, SdfMaterial, SdfNode, SdfObject, SdfProperty, SdfPropertyValue, SdfShape3d, SdfState,
    SdfStep, SdfStepMaterial,
};
use ty_math::TyVector3F64;

/// A model at one meter per voxel whose root part `model` adds the 4x2x4
/// `body` from the origin. The part `lid` turns about `[2, 2, 2]` and adds the
/// 2x1x2 `lid` on top of the body. The part `pebble` adds one cell 2 cells
/// past the body. The body and the pebble take identical materials, and the
/// lid's material sets the custom property `tag`.
pub fn parts_main() -> SdfMain {
    let cuboid = |min: [f64; 3], max: [f64; 3]| SdfShape3d::Box {
        min: TyVector3F64::from_array(min),
        max: TyVector3F64::from_array(max),
        round: None,
    };

    let add = |name: &str, shape: u32, material: u32| SdfStep::Add {
        name: name.to_string(),
        shape_id: U32Id::from_u32(shape),
        material: SdfStepMaterial::Material(U32Id::from_u32(material)),
    };

    let material = |tag: Option<&str>| SdfMaterial::Material {
        properties: [Some(("baseColor", "#336699")), tag.map(|tag| ("tag", tag))]
            .into_iter()
            .flatten()
            .map(|(name, value)| SdfProperty {
                name: name.to_string(),
                value: SdfPropertyValue::Text(value.to_string()),
            })
            .collect(),
    };

    let object = |name: &str, step: u32| SdfObject {
        name: name.to_string(),
        step_ids: vec![U32Id::from_u32(step)],
    };

    let node = |name: &str, pivot: [f64; 3], object: u32, nodes: Vec<u32>| SdfNode {
        name: name.to_string(),
        pivot: Some(TyVector3F64::from_array(pivot)),
        offset: None,
        rotation: Vec::new(),
        joint: false,
        child_object_ids: vec![U32Id::from_u32(object)],
        child_node_ids: nodes.into_iter().map(U32Id::from_u32).collect(),
    };

    SdfMain::new(SdfState {
        shapes3d: IdVec::from_vec(vec![
            cuboid([0.0, 0.0, 0.0], [4.0, 2.0, 4.0]),
            cuboid([1.0, 2.0, 1.0], [3.0, 3.0, 3.0]),
            cuboid([6.0, 0.0, 0.0], [7.0, 1.0, 1.0]),
        ]),
        materials: IdVec::from_vec(vec![material(None), material(Some("lid")), material(None)]),
        steps: IdVec::from_vec(vec![
            add("body", 0, 0),
            add("lid", 1, 1),
            add("pebble", 2, 2),
        ]),
        objects: IdVec::from_vec(vec![
            object("model", 0),
            object("lid", 1),
            object("pebble", 2),
        ]),
        nodes: IdVec::from_vec(vec![
            node("lid", [2.0, 2.0, 2.0], 1, Vec::new()),
            node("pebble", [6.0, 0.0, 0.0], 2, Vec::new()),
            node("model", [0.0, 0.0, 0.0], 0, vec![0, 1]),
        ]),
        root_node_ids: vec![U32Id::from_u32(2)],
        ..SdfState::default()
    })
    .unwrap()
}
