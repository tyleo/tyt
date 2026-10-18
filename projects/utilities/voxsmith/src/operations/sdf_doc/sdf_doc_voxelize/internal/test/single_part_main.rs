use branded_id::{IdVec, U32Id};
use sdfcore::{SdfMain, SdfMaterial, SdfNode, SdfObject, SdfShape3d, SdfState, SdfStep};

/// A model named `model` whose root part runs `steps` over `shapes3d`, with
/// two materials.
pub fn single_part_main(shapes3d: Vec<SdfShape3d>, steps: Vec<SdfStep>) -> SdfMain {
    let step_ids = (0..steps.len() as u32).map(U32Id::from_u32).collect();

    SdfMain::new(SdfState {
        shapes3d: IdVec::from_vec(shapes3d),
        materials: IdVec::from_vec(vec![
            SdfMaterial::Material {
                properties: Vec::new(),
            };
            2
        ]),
        steps: IdVec::from_vec(steps),
        objects: IdVec::from_vec(vec![SdfObject {
            name: "model".to_string(),
            step_ids,
        }]),
        nodes: IdVec::from_vec(vec![SdfNode {
            name: "model".to_string(),
            pivot: None,
            offset: None,
            rotation: Vec::new(),
            child_object_ids: vec![U32Id::from_u32(0)],
            child_node_ids: Vec::new(),
        }]),
        root_node_ids: vec![U32Id::from_u32(0)],
        ..SdfState::default()
    })
    .unwrap()
}
