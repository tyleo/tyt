use crate::{Error, Result, operations::node::node_name};
use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

/// Places each node in `node_ids` under node `parent_id`, after its existing
/// child nodes, or at the end of the roots when `parent_id` is `None`. Errors,
/// changing nothing, when an id is not one of the main's, the edge exists, or
/// an edge would close a cycle.
pub fn link_nodes<T: VoxExt>(
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
            if root_ids.contains(&node_id) {
                return Err(Error::invalid(format!(
                    "node \"{}\" is already a root",
                    node_name(main, node_id)
                )));
            }

            root_ids.push(node_id);
        }

        return Ok(main.set_root_hierarchy_node_ids(root_ids)?);
    };

    let Some(parent) = main.hierarchy_node(parent_id) else {
        return Err(VoxError::UnknownHierarchyNode { node_id: parent_id }.into());
    };

    let mut child_node_ids = parent.child_node_ids.clone();

    for &node_id in node_ids {
        if child_node_ids.contains(&node_id) {
            return Err(Error::invalid(format!(
                "node \"{}\" already has child node \"{}\"",
                parent.name,
                node_name(main, node_id)
            )));
        }

        child_node_ids.push(node_id);
    }

    let child_object_ids = parent.child_object_ids.clone();

    let parent_name = parent.name.clone();

    match main.set_hierarchy_node_children(parent_id, child_node_ids, child_object_ids) {
        Err(VoxError::Cycle { .. }) => Err(Error::invalid(format!(
            "linking under node \"{parent_name}\" would close a cycle, because it is a selected \
             node or below one"
        ))),

        result => Ok(result?),
    }
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::link_nodes, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use voxcore::{BVoxHierarchyNode, VoxHierarchyNode, VoxMain};

    fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    /// Root `house` with child `door`, root `garage`, and unplaced `shed`.
    /// Ids run door 0, house 1, garage 2, shed 3.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let node = |name: &str, child_node_ids: Vec<U32Id<BVoxHierarchyNode>>| VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids,
            ..Default::default()
        };

        let door_id = main.retain_hierarchy_node(node("door", vec![])).unwrap();
        let house_id = main
            .retain_hierarchy_node(node("house", vec![door_id]))
            .unwrap();
        let garage_id = main.retain_hierarchy_node(node("garage", vec![])).unwrap();
        main.retain_hierarchy_node(node("shed", vec![])).unwrap();

        main.set_root_hierarchy_node_ids(vec![house_id, garage_id])
            .unwrap();

        main.put_ext(HookRecorder::default())
    }

    fn child_node_ids(
        main: &VoxMain<HookRecorder>,
        node_id: U32Id<BVoxHierarchyNode>,
    ) -> &[U32Id<BVoxHierarchyNode>] {
        &main.hierarchy_node(node_id).unwrap().child_node_ids
    }

    #[test]
    fn a_parent_appends_the_nodes_to_its_children() {
        let mut main = scene();

        link_nodes(
            &mut main,
            &[U32Id::from_u32(0), U32Id::from_u32(3)],
            Some(U32Id::from_u32(2)),
        )
        .unwrap();

        assert_eq!(child_node_ids(&main, node_id(2)), [node_id(0), node_id(3)]);
        assert_eq!(child_node_ids(&main, node_id(1)), [node_id(0)]);
        assert_eq!(HookRecorder::events(&main), ["node 2 children set"]);
        main.validate().unwrap();
    }

    #[test]
    fn no_parent_appends_the_nodes_to_the_roots() {
        let mut main = scene();

        link_nodes(&mut main, &[U32Id::from_u32(3)], None).unwrap();

        assert_eq!(
            main.root_hierarchy_node_ids(),
            [U32Id::from_u32(1), U32Id::from_u32(2), U32Id::from_u32(3)]
        );
        assert_eq!(HookRecorder::events(&main), ["roots set"]);
    }

    #[test]
    fn an_existing_edge_is_an_error_that_changes_nothing() {
        let mut main = scene();

        assert!(link_nodes(&mut main, &[U32Id::from_u32(0)], Some(U32Id::from_u32(1))).is_err());
        assert!(link_nodes(&mut main, &[U32Id::from_u32(2)], None).is_err());
        assert!(HookRecorder::events(&main).is_empty());
    }

    #[test]
    fn a_cycle_is_an_error_that_changes_nothing() {
        let mut main = scene();

        // House under door, then house under itself.
        for parent_id in [0, 1] {
            let error = link_nodes(
                &mut main,
                &[U32Id::from_u32(1)],
                Some(U32Id::from_u32(parent_id)),
            )
            .unwrap_err();

            assert!(error.to_string().contains("cycle"), "{error}");
        }

        assert!(HookRecorder::events(&main).is_empty());
    }
}
