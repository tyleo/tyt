use crate::{Result, utilities::IndexRange};
use branded_id::U32Id;
use pathspec::{GitIgnoreRegex, is_file_path_match};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxExt, VoxMain};

/// A hierarchy-node id.
type NodeId = U32Id<BVoxHierarchyNode>;

/// An object id.
type ObjectId = U32Id<BVoxObject>;

/// Resolves object selectors against `main` to the ids of the matching
/// objects, in document order and deduplicated. An index selector picks
/// objects by position. A path glob matches hierarchy paths with the gitignore
/// engine, where a matched node contributes its whole subtree and a matched
/// object contributes itself. With neither selector every object matches. How
/// many matches are acceptable is the caller's policy.
pub fn select_objects<T: VoxExt>(
    main: &VoxMain<T>,
    select: &[String],
    select_index: &[IndexRange],
) -> Result<Vec<U32Id<BVoxObject>>> {
    let object_ids: Vec<ObjectId> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .collect();

    if select.is_empty() && select_index.is_empty() {
        return Ok(object_ids);
    }

    let mut chosen = vec![false; object_ids.len()];

    for (object_index, chosen_flag) in chosen.iter_mut().enumerate() {
        if select_index
            .iter()
            .any(|selector| selector.contains(object_index))
        {
            *chosen_flag = true;
        }
    }

    if !select.is_empty() {
        select_by_path(main, &object_ids, select, &mut chosen)?;
    }

    Ok(object_ids
        .into_iter()
        .zip(chosen)
        .filter_map(|(object_id, chosen)| chosen.then_some(object_id))
        .collect())
}

/// Marks every object a path glob reaches. Each object's placement paths are
/// tested against the gitignore patterns with [`is_file_path_match`], which
/// pulls an object in when its own path matches or when a selected ancestor
/// node does, so a matched node selects its whole subtree. Matching any of a
/// DAG object's placement paths selects it.
fn select_by_path<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[ObjectId],
    select: &[String],
    chosen: &mut [bool],
) -> Result<()> {
    let patterns = GitIgnoreRegex::from_spans_ignore_inert(select)?;

    let node_paths = node_paths(main);

    let object_paths = object_paths(main, object_ids, &node_paths);

    for (object_index, path) in &object_paths {
        // Objects are file leaves; the ancestor walk in `is_file_path_match` is
        // what carries a selected node down to its objects.
        if is_file_path_match(&patterns, path) == Some(true) {
            chosen[*object_index] = true;
        }
    }

    Ok(())
}

/// Every hierarchy node's path strings, one per placement (a node reached
/// through several parents gets one path each), a chain of node names from a
/// root.
fn node_paths<T: VoxExt>(main: &VoxMain<T>) -> Vec<(NodeId, String)> {
    let mut paths = Vec::new();

    let mut stack = Vec::new();

    for &root_id in main.root_hierarchy_node_ids() {
        walk_node_paths(main, root_id, "", &mut stack, &mut paths);
    }

    paths
}

/// Records `node_id`'s path (built from `prefix`), then recurses into its child
/// nodes. `stack` is the current root-to-node chain and guards against a cycle.
fn walk_node_paths<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: NodeId,
    prefix: &str,
    stack: &mut Vec<NodeId>,
    paths: &mut Vec<(NodeId, String)>,
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

    paths.push((node_id, path.clone()));

    stack.push(node_id);

    for &child_id in &node.child_node_ids {
        walk_node_paths(main, child_id, &path, stack, paths);
    }

    stack.pop();
}

/// Every object's path strings, one per placement (a placing node's path plus
/// the object name), or the bare object name when no node places it. Each entry
/// pairs the object's index in `object_ids` with a path.
fn object_paths<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[ObjectId],
    node_paths: &[(NodeId, String)],
) -> Vec<(usize, String)> {
    let mut placed = vec![false; object_ids.len()];

    let mut paths = Vec::new();

    for (node_id, node_path) in node_paths {
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

#[cfg(test)]
mod tests {
    use crate::utilities::{
        IndexRange,
        select_objects::{NodeId, ObjectId, select_objects},
    };
    use ty_math::TyVector3U32;
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject};

    /// Adds an empty named object and returns its id.
    fn object_id(main: &mut VoxMain, name: &str) -> ObjectId {
        let object = VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        main.retain_object(object).unwrap()
    }

    /// Adds a hierarchy node placing the given child nodes and objects.
    fn node_id(
        main: &mut VoxMain,
        name: &str,
        child_node_ids: Vec<NodeId>,
        child_object_ids: Vec<ObjectId>,
    ) -> NodeId {
        let node = VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids,
            child_object_ids,
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap()
    }

    /// The path globs as the owned `String`s the resolver takes.
    fn globs(patterns: &[&str]) -> Vec<String> {
        patterns.iter().map(|pattern| pattern.to_string()).collect()
    }

    /// The single-index selector for `index`.
    fn index(index: usize) -> IndexRange {
        IndexRange::new(index, index).unwrap()
    }

    #[test]
    fn no_selectors_select_every_object() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        assert_eq!(select_objects(&main, &[], &[]).unwrap(), vec![a_id, b_id]);
    }

    #[test]
    fn select_index_picks_by_position() {
        let mut main = VoxMain::default();

        object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        assert_eq!(select_objects(&main, &[], &[index(1)]).unwrap(), vec![b_id]);
    }

    #[test]
    fn a_glob_matches_an_object_by_its_path() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        let root_id = node_id(&mut main, "root", vec![], vec![a_id, b_id]);
        main.push_root_hierarchy_node_id(root_id).unwrap();

        assert_eq!(
            select_objects(&main, &globs(&["root/b"]), &[]).unwrap(),
            vec![b_id]
        );
    }

    #[test]
    fn a_glob_on_a_node_selects_its_subtree() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        let group_id = node_id(&mut main, "group", vec![], vec![a_id, b_id]);
        main.push_root_hierarchy_node_id(group_id).unwrap();

        assert_eq!(
            select_objects(&main, &globs(&["group"]), &[]).unwrap(),
            vec![a_id, b_id]
        );
    }

    #[test]
    fn a_negation_prunes_a_subtree() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        let keep_id = node_id(&mut main, "keep", vec![], vec![a_id]);
        let drop_id = node_id(&mut main, "drop", vec![], vec![b_id]);
        let root_id = node_id(&mut main, "root", vec![keep_id, drop_id], vec![]);
        main.push_root_hierarchy_node_id(root_id).unwrap();

        // Select everything under root, then subtract the `drop` branch;
        // last-match-wins leaves only `a`.
        assert_eq!(
            select_objects(&main, &globs(&["root/**", "!drop/"]), &[]).unwrap(),
            vec![a_id]
        );
    }

    #[test]
    fn a_nonmatching_glob_selects_nothing() {
        let mut main = VoxMain::default();

        object_id(&mut main, "a");

        assert_eq!(
            select_objects(&main, &globs(&["nope"]), &[]).unwrap(),
            Vec::<ObjectId>::new(),
        );
    }

    #[test]
    fn selectors_union_and_deduplicate() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");
        let c_id = object_id(&mut main, "c");

        let root_id = node_id(&mut main, "root", vec![], vec![a_id, b_id, c_id]);
        main.push_root_hierarchy_node_id(root_id).unwrap();

        // A `root/**` path glob selects every object under root; an index-0
        // selector re-selects `a`, which still appears once, in document order.
        assert_eq!(
            select_objects(&main, &globs(&["root/**"]), &[index(0)]).unwrap(),
            vec![a_id, b_id, c_id],
        );
    }
}
