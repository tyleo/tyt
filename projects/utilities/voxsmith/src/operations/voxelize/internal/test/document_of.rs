use meshdoc::{MeshHierarchyNode, MeshMain, MeshObject, MeshPrimitive};
use ty_math::TyTransformF64;

/// `main` with one object of `primitive` under one root node named
/// `node_name` and placed at `transform`, validated.
pub(crate) fn document_of(
    mut main: MeshMain<()>,
    primitive: MeshPrimitive,
    node_name: Option<&str>,
    transform: TyTransformF64,
) -> MeshMain<()> {
    let mut object = MeshObject::new(String::new());
    object.retain_primitive(primitive);
    let object_id = main.retain_object(object).unwrap();

    let node_id = main
        .retain_hierarchy_node(MeshHierarchyNode {
            name: node_name.unwrap_or_default().to_owned(),
            transform,
            child_object_ids: vec![object_id],
            ..Default::default()
        })
        .unwrap();
    main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
    main.validate().unwrap();
    main
}
