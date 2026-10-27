use crate::{Error, Result, operations::node::node_name};
use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

/// Removes each node in `node_ids` from node `parent_id`'s child nodes, or
/// from the roots when `parent_id` is `None`. A node left with no parents
/// stays in the document unplaced. Errors, changing nothing, when an id is not
/// one of the main's or the edge does not exist.
pub fn unlink_nodes<T: VoxExt>(
    main: &mut VoxMain<T>,
    node_ids: &[NodeId],
    parent_id: Option<NodeId>,
) -> Result<()> {
    for &node_id in node_ids {
        if main.hierarchy_node(node_id).is_none() {
            return Err(VoxError::UnknownHierarchyNode { node_id }.into());
        }
    }

    let Some(parent_id) = parent_id else {
        let mut root_ids = main.root_hierarchy_node_ids().to_vec();

        for &node_id in node_ids {
            let Some(index) = root_ids.iter().position(|&root_id| root_id == node_id) else {
                return Err(Error::invalid(format!(
                    "node \"{}\" is not a root",
                    node_name(main, node_id)
                )));
            };

            root_ids.remove(index);
        }

        return Ok(main.set_root_hierarchy_node_ids(root_ids)?);
    };

    let Some(parent) = main.hierarchy_node(parent_id) else {
        return Err(VoxError::UnknownHierarchyNode { node_id: parent_id }.into());
    };

    let mut child_node_ids = parent.child_node_ids.clone();

    for &node_id in node_ids {
        let Some(index) = child_node_ids
            .iter()
            .position(|&child_id| child_id == node_id)
        else {
            return Err(Error::invalid(format!(
                "node \"{}\" has no child node \"{}\"",
                parent.name,
                node_name(main, node_id)
            )));
        };

        child_node_ids.remove(index);
    }

    let child_object_ids = parent.child_object_ids.clone();

    Ok(main.set_hierarchy_node_children(parent_id, child_node_ids, child_object_ids)?)
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::unlink_nodes, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use voxcore::{VoxHierarchyNode, VoxMain};

    /// Roots `house` and `garage`, both placing `door`. Ids run door 0,
    /// house 1, garage 2.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let door = VoxHierarchyNode {
            name: "door".to_owned(),
            ..Default::default()
        };

        let door_id = main.retain_hierarchy_node(door).unwrap();

        for name in ["house", "garage"] {
            let node = VoxHierarchyNode {
                name: name.to_owned(),
                child_node_ids: vec![door_id],
                ..Default::default()
            };

            let node_id = main.retain_hierarchy_node(node).unwrap();
            main.push_root_hierarchy_node_id(node_id).unwrap();
        }

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn a_parent_drops_the_edge_and_the_node_stays() {
        let mut main = scene();

        unlink_nodes(&mut main, &[U32Id::from_u32(0)], Some(U32Id::from_u32(1))).unwrap();

        assert!(
            main.hierarchy_node(U32Id::from_u32(1))
                .unwrap()
                .child_node_ids
                .is_empty()
        );
        assert_eq!(main.hierarchy_node_count(), 3);
        assert_eq!(HookRecorder::events(&main), ["node 1 children set"]);
        main.validate().unwrap();
    }

    #[test]
    fn no_parent_drops_the_nodes_from_the_roots() {
        let mut main = scene();

        unlink_nodes(&mut main, &[U32Id::from_u32(2)], None).unwrap();

        assert_eq!(main.root_hierarchy_node_ids(), [U32Id::from_u32(1)]);
        assert_eq!(HookRecorder::events(&main), ["roots set"]);
    }

    #[test]
    fn a_missing_edge_is_an_error_that_changes_nothing() {
        let mut main = scene();

        assert!(unlink_nodes(&mut main, &[U32Id::from_u32(1)], Some(U32Id::from_u32(2))).is_err());
        assert!(unlink_nodes(&mut main, &[U32Id::from_u32(0)], None).is_err());
        assert!(HookRecorder::events(&main).is_empty());
    }
}
