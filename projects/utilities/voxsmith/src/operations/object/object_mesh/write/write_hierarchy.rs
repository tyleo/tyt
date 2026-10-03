use crate::{Result, utilities::placing_nodes};
use branded_id::{U32Id, ext::IteratorExt};
use meshdoc::{BMeshHierarchyNode, BMeshObject, MeshHierarchyNode, MeshMain};
use std::collections::{HashMap, HashSet};
use ty_math::TyTransformF64;
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// Places `objects`, each a meshed object of `main` beside its mesh object, by
/// the hierarchy of `main` narrowed to the nodes reaching one of them, with
/// node positions scaled by `voxel_size` onto meters. An object no node places
/// gets a root node of its name after the mirrored roots. `document` holds no
/// node yet.
pub fn write_hierarchy<T: VoxExt>(
    document: &mut MeshMain<()>,
    main: &VoxMain<T>,
    objects: &[(U32Id<BVoxObject>, U32Id<BMeshObject>)],
    voxel_size: f64,
) -> Result<()> {
    assert_eq!(
        document.hierarchy_node_count(),
        0,
        "the hierarchy is written once into an empty document"
    );

    let mesh_object_ids: HashMap<_, _> = objects.iter().copied().collect();

    let placing = placing_nodes(main, &mesh_object_ids.keys().copied().collect());

    let mesh_node_ids: HashMap<_, U32Id<BMeshHierarchyNode>> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .filter(|node_id| placing.contains(node_id))
        .enumerate_ids()
        .map(|(mesh_node_id, node_id)| (node_id, mesh_node_id))
        .collect();

    let mut placed = HashSet::new();

    let nodes = main
        .iter_hierarchy_nodes()
        .filter(|(node_id, _)| placing.contains(node_id))
        .map(|(_, node)| {
            let child_object_ids = node
                .child_object_ids
                .iter()
                .filter_map(|object_id| mesh_object_ids.get(object_id))
                .copied()
                .collect::<Vec<_>>();

            placed.extend(child_object_ids.iter().copied());

            MeshHierarchyNode {
                name: node.name.clone(),
                transform: TyTransformF64 {
                    position: node.transform.position * voxel_size,
                    ..node.transform
                },
                child_node_ids: node
                    .child_node_ids
                    .iter()
                    .filter_map(|child_id| mesh_node_ids.get(child_id))
                    .copied()
                    .collect(),
                child_object_ids,
            }
        })
        .collect();

    let node_ids = document.retain_hierarchy_nodes(nodes)?;

    debug_assert!(
        node_ids
            .iter()
            .enumerate()
            .all(|(index, node_id)| node_id.to_u32() as usize == index),
        "the batch took the predicted ids"
    );

    let mut root_ids: Vec<_> = main
        .root_hierarchy_node_ids()
        .iter()
        .filter_map(|root_id| mesh_node_ids.get(root_id))
        .copied()
        .collect();

    for &(_, mesh_object_id) in objects {
        if placed.contains(&mesh_object_id) {
            continue;
        }

        let name = document
            .object(mesh_object_id)
            .expect("a placed object is one of the document's")
            .name()
            .to_owned();

        root_ids.push(document.retain_hierarchy_node(MeshHierarchyNode {
            name,
            child_object_ids: vec![mesh_object_id],
            ..Default::default()
        })?);
    }

    document.set_root_hierarchy_node_ids(root_ids)?;

    Ok(())
}
