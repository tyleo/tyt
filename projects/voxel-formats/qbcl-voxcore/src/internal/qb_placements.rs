use crate::QbPlacement;
use branded_id::U32Id;
use std::collections::HashSet;
use ty_math::TyVector3I32;
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxMain};

/// The matrices the scene flattens to, one per object placement in hierarchy
/// order. A placement lands at the world translation summed down from the
/// roots and rounded to whole voxels. An object no node places lands once at
/// the origin. A node's first object takes the node's name. The rest take
/// the object's name.
pub fn qb_placements<T>(state: &VoxMain<T>) -> Vec<QbPlacement> {
    let mut placements = Vec::new();
    for &root_id in state.root_hierarchy_node_ids() {
        push_node_placements(state, root_id, TyVector3I32::new(0, 0, 0), &mut placements);
    }

    let placed: HashSet<U32Id<BVoxObject>> = placements
        .iter()
        .map(|placement| placement.object_id)
        .collect();
    for (object_id, object) in state.iter_objects() {
        if !placed.contains(&object_id) {
            placements.push(QbPlacement {
                object_id,
                name: object.name().to_owned(),
                position: [0, 0, 0],
            });
        }
    }

    placements
}

/// Walks `node_id` and its subtree. The translations sum into the world
/// position each placement carries.
fn push_node_placements<T>(
    state: &VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent: TyVector3I32,
    placements: &mut Vec<QbPlacement>,
) {
    let node = state
        .hierarchy_node(node_id)
        .expect("a hierarchy id from the state resolves");
    let world = parent + node.transform.position.round().as_ivec3();

    for (index, &object_id) in node.child_object_ids.iter().enumerate() {
        let object = state
            .object(object_id)
            .expect("a placed object is one of the state's");
        let name = if index == 0 {
            node.name.clone()
        } else {
            object.name().to_owned()
        };
        placements.push(QbPlacement {
            object_id,
            name,
            position: world.to_array(),
        });
    }

    for &child_id in &node.child_node_ids {
        push_node_placements(state, child_id, world, placements);
    }
}
