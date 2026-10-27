use crate::{
    Result,
    utilities::{IdSelector, NodePath, node_paths},
};
use branded_id::{RangeInclusiveExt, U32Id};
use pathspec::{GitIgnoreRegex, is_file_path_match};
use voxcore::{BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// An object id.
type ObjectId = U32Id<BVoxObject>;

/// Resolves object selectors against `main` to the ids of the matching
/// objects, in document order and deduplicated. An id selector picks objects
/// by id and errors on an id `main` lacks. A path glob matches hierarchy paths
/// with the gitignore engine, where a matched node contributes its whole
/// subtree and a matched object contributes itself. With neither selector
/// every object matches. How many matches are acceptable is the caller's
/// policy.
pub fn select_objects<T: VoxExt>(
    main: &VoxMain<T>,
    select: &[String],
    select_index: &[IdSelector<BVoxObject>],
) -> Result<Vec<U32Id<BVoxObject>>> {
    for selector in select_index {
        let Some(range) = selector.as_range() else {
            continue;
        };

        for object_id in range.clone().into_id_range() {
            if main.object(object_id).is_none() {
                return Err(VoxError::UnknownObject { object_id }.into());
            }
        }
    }

    let object_ids: Vec<ObjectId> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .collect();

    if select.is_empty() && select_index.is_empty() {
        return Ok(object_ids);
    }

    let mut chosen: Vec<bool> = object_ids
        .iter()
        .map(|&object_id| {
            select_index
                .iter()
                .any(|selector| selector.contains(object_id))
        })
        .collect();

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

/// Every object's path strings, one per placement (a placing node's path plus
/// the object name), or the bare object name when no node places it. Each entry
/// pairs the object's index in `object_ids` with a path.
fn object_paths<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &[ObjectId],
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

#[cfg(test)]
mod tests {
    use crate::utilities::{
        IdSelector,
        select_objects::{ObjectId, select_objects},
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxHierarchyNode, VoxHierarchyNode, VoxMain, VoxObject};

    /// A hierarchy-node id.
    type NodeId = U32Id<BVoxHierarchyNode>;

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

    #[test]
    fn no_selectors_select_every_object() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        assert_eq!(select_objects(&main, &[], &[]).unwrap(), vec![a_id, b_id]);
    }

    #[test]
    fn select_index_picks_by_id() {
        let mut main = VoxMain::default();

        object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        assert_eq!(
            select_objects(&main, &[], &[IdSelector::id(b_id)]).unwrap(),
            vec![b_id]
        );
    }

    #[test]
    fn select_index_errors_on_an_id_the_main_lacks() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        main.release_object(b_id).unwrap();

        assert_eq!(
            select_objects(&main, &[], &[IdSelector::id(b_id)])
                .unwrap_err()
                .to_string(),
            "object 1 is not one of this state's"
        );

        let range = IdSelector::range(a_id..=b_id).unwrap();

        assert_eq!(
            select_objects(&main, &[], &[range])
                .unwrap_err()
                .to_string(),
            "object 1 is not one of this state's"
        );
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
    fn an_unplaced_node_carries_its_objects_path() {
        let mut main = VoxMain::default();

        let a_id = object_id(&mut main, "a");

        node_id(&mut main, "loose", vec![], vec![a_id]);

        assert_eq!(
            select_objects(&main, &globs(&["loose"]), &[]).unwrap(),
            vec![a_id]
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

        // A `root/**` path glob selects every object under root; an id
        // selector re-selects `a`, which still appears once, in document order.
        assert_eq!(
            select_objects(&main, &globs(&["root/**"]), &[IdSelector::id(a_id)]).unwrap(),
            vec![a_id, b_id, c_id],
        );
    }
}
