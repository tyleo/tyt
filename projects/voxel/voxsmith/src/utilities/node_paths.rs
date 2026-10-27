use crate::utilities::NodePath;
use branded_id::U32Id;
use std::collections::HashSet;
use ty_math::TyTransformF64;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

/// Every hierarchy node's paths, each a chain of node names from a root with
/// the world transform the chain composes. A node reached through several
/// parents gets one path per placement. A node that neither the roots nor any
/// node lists starts its own chain, the way an unplaced object's path is its
/// bare name.
pub fn node_paths<T: VoxExt>(main: &VoxMain<T>) -> Vec<NodePath> {
    let mut paths = Vec::new();

    let mut stack = Vec::new();

    for &root_id in main.root_hierarchy_node_ids() {
        walk_node_paths(
            main,
            root_id,
            "",
            &TyTransformF64::IDENTITY,
            &mut stack,
            &mut paths,
        );
    }

    let listed: HashSet<NodeId> = main
        .root_hierarchy_node_ids()
        .iter()
        .copied()
        .chain(
            main.iter_hierarchy_nodes()
                .flat_map(|(_, node)| node.child_node_ids.iter().copied()),
        )
        .collect();

    let unplaced_ids: Vec<NodeId> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .filter(|node_id| !listed.contains(node_id))
        .collect();

    for node_id in unplaced_ids {
        walk_node_paths(
            main,
            node_id,
            "",
            &TyTransformF64::IDENTITY,
            &mut stack,
            &mut paths,
        );
    }

    paths
}

/// Records `node_id`'s path and world transform, extending `prefix` and
/// `parent`, the chain above it, then recurses into its child nodes. `stack`
/// is the current root-to-node chain and guards against a cycle.
fn walk_node_paths<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: NodeId,
    prefix: &str,
    parent: &TyTransformF64,
    stack: &mut Vec<NodeId>,
    paths: &mut Vec<NodePath>,
) {
    if stack.contains(&node_id) {
        return;
    }

    let Some(node) = main.hierarchy_node(node_id) else {
        return;
    };

    let path = if prefix.is_empty() {
        node.name.clone()
    } else {
        format!("{prefix}/{}", node.name)
    };

    let world = parent.compose(&node.transform);

    paths.push(NodePath {
        node_id,
        path: path.clone(),
        world,
    });

    stack.push(node_id);

    for &child_id in &node.child_node_ids {
        walk_node_paths(main, child_id, &path, &world, stack, paths);
    }

    stack.pop();
}

#[cfg(test)]
mod tests {
    use crate::utilities::{
        NodePath,
        node_paths::{NodeId, node_paths},
    };
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxcore::{VoxHierarchyNode, VoxMain};

    fn node_id(main: &mut VoxMain, name: &str, child_node_ids: Vec<NodeId>) -> NodeId {
        let node = VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids,
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap()
    }

    fn paths(main: &VoxMain) -> Vec<String> {
        node_paths(main).into_iter().map(|path| path.path).collect()
    }

    #[test]
    fn a_shared_node_gets_one_path_per_placement() {
        let mut main: VoxMain = VoxMain::default();

        let shared_id = node_id(&mut main, "shared", vec![]);
        let left_id = node_id(&mut main, "left", vec![shared_id]);
        let right_id = node_id(&mut main, "right", vec![shared_id]);
        main.push_root_hierarchy_node_id(left_id).unwrap();
        main.push_root_hierarchy_node_id(right_id).unwrap();

        assert_eq!(
            paths(&main),
            ["left", "left/shared", "right", "right/shared"]
        );
    }

    #[test]
    fn an_unplaced_node_starts_its_own_chain() {
        let mut main: VoxMain = VoxMain::default();

        let child_id = node_id(&mut main, "child", vec![]);
        node_id(&mut main, "loose", vec![child_id]);
        let root_id = node_id(&mut main, "root", vec![]);
        main.push_root_hierarchy_node_id(root_id).unwrap();

        assert_eq!(paths(&main), ["root", "loose", "loose/child"]);
    }

    #[test]
    fn a_path_composes_the_transforms_above_it() {
        let mut main: VoxMain = VoxMain::default();

        let child_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "child".to_owned(),
                transform: TyTransformF64::from_translation(TyVector3F64::new(0.0, 0.0, 1.0)),
                ..Default::default()
            })
            .unwrap();

        let root_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "root".to_owned(),
                transform: TyTransformF64::new(
                    TyVector3F64::new(10.0, 0.0, 0.0),
                    TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
                    TyVector3F64::splat(2.0),
                ),
                child_node_ids: vec![child_id],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(root_id).unwrap();

        let [root, child] = node_paths(&main).try_into().unwrap();

        assert_eq!(
            root,
            NodePath {
                node_id: root_id,
                path: "root".to_owned(),
                world: main.hierarchy_node(root_id).unwrap().transform,
            }
        );

        // The child's local +Z turns to world +X and doubles.
        assert_eq!(child.node_id, child_id);
        assert_eq!(child.path, "root/child");
        assert!(
            (child.world.position - TyVector3F64::new(12.0, 0.0, 0.0)).length() < 1e-9,
            "{:?}",
            child.world.position
        );
        assert_eq!(child.world.scale, TyVector3F64::splat(2.0));
    }
}
