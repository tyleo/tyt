use crate::Result;
use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxHierarchyNode, VoxMain};

/// Appends an empty node named `name` with an identity transform and returns
/// its id. It goes after `parent_id`'s child nodes, or at the end of the roots
/// when `parent_id` is `None`. Errors, changing nothing, when `parent_id`
/// is not one of the main's nodes.
pub fn add_node<T: VoxExt>(
    main: &mut VoxMain<T>,
    name: String,
    parent_id: Option<U32Id<BVoxHierarchyNode>>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    if let Some(parent_id) = parent_id
        && main.hierarchy_node(parent_id).is_none()
    {
        return Err(VoxError::UnknownHierarchyNode { node_id: parent_id }.into());
    }

    let node = VoxHierarchyNode {
        name,
        ..Default::default()
    };

    let node_id = main.retain_hierarchy_node(node)?;

    let Some(parent_id) = parent_id else {
        main.push_root_hierarchy_node_id(node_id)?;

        return Ok(node_id);
    };

    let parent = main
        .hierarchy_node(parent_id)
        .expect("parent_id is checked above");

    let mut child_node_ids = parent.child_node_ids.clone();
    child_node_ids.push(node_id);

    let child_object_ids = parent.child_object_ids.clone();

    main.set_hierarchy_node_children(parent_id, child_node_ids, child_object_ids)?;

    Ok(node_id)
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::add_node, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::TyTransformF64;
    use voxcore::{VoxHierarchyNode, VoxMain};

    /// A root `house` with one child node `door`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let door = VoxHierarchyNode {
            name: "door".to_owned(),
            ..Default::default()
        };

        let door_id = main.retain_hierarchy_node(door).unwrap();

        let house = VoxHierarchyNode {
            name: "house".to_owned(),
            child_node_ids: vec![door_id],
            ..Default::default()
        };

        let house_id = main.retain_hierarchy_node(house).unwrap();
        main.push_root_hierarchy_node_id(house_id).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn a_node_without_a_parent_becomes_the_last_root() {
        let mut main = scene();

        let node_id = add_node(&mut main, "garage".to_owned(), None).unwrap();

        let node = main.hierarchy_node(node_id).unwrap();

        assert_eq!(node.name, "garage");
        assert_eq!(node.transform, TyTransformF64::IDENTITY);
        assert_eq!(
            main.root_hierarchy_node_ids(),
            [U32Id::from_u32(1), node_id]
        );
        assert_eq!(
            HookRecorder::events(&main),
            ["node 2 retained", "roots set"]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_parent_places_the_node_after_its_child_nodes() {
        let mut main = scene();

        let node_id = add_node(&mut main, "window".to_owned(), Some(U32Id::from_u32(1))).unwrap();

        assert_eq!(
            main.hierarchy_node(U32Id::from_u32(1))
                .unwrap()
                .child_node_ids,
            [U32Id::from_u32(0), node_id]
        );
        assert_eq!(main.root_hierarchy_node_ids(), [U32Id::from_u32(1)]);
        assert_eq!(
            HookRecorder::events(&main),
            ["node 2 retained", "node 1 children set"]
        );
        main.validate().unwrap();
    }

    #[test]
    fn an_unknown_parent_is_an_error_that_changes_nothing() {
        let mut main = scene();

        assert!(add_node(&mut main, "window".to_owned(), Some(U32Id::from_u32(9))).is_err());
        assert_eq!(main.hierarchy_node_count(), 2);
        assert!(HookRecorder::events(&main).is_empty());
    }
}
