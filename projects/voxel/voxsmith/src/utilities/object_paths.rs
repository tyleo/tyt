use crate::utilities::NodePath;
use branded_id::U32Id;
use voxcore::{BVoxObject, VoxExt, VoxMain};

/// Every object's path strings, one per placement (a placing node's path plus
/// the object name), or the bare object name when no node places it. Each entry
/// pairs the object's index in `object_ids` with a path.
pub fn object_paths<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    node_paths: &[NodePath],
) -> Vec<(usize, String)> {
    let mut placed = vec![false; object_ids.len()];

    let mut paths = Vec::new();

    for NodePath {
        node_id,
        path: node_path,
        ..
    } in node_paths
    {
        let Some(node) = main.hierarchy_node(*node_id) else {
            continue;
        };

        for &child_object_id in &node.child_object_ids {
            let Some(object_index) = object_ids
                .iter()
                .position(|&object_id| object_id == child_object_id)
            else {
                continue;
            };

            let Some(object) = main.object(child_object_id) else {
                continue;
            };

            placed[object_index] = true;

            paths.push((object_index, format!("{node_path}/{}", object.name())));
        }
    }

    for (object_index, &object_id) in object_ids.iter().enumerate() {
        if placed[object_index] {
            continue;
        }

        if let Some(object) = main.object(object_id) {
            paths.push((object_index, object.name().to_owned()));
        }
    }

    paths
}
