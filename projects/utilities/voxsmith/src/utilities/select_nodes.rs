use crate::{
    Result,
    utilities::{IndexRange, NodePath, is_node_path_match, node_paths},
};
use branded_id::U32Id;
use pathspec::GitIgnoreRegex;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

/// Resolves node selectors against `main` to the matching hierarchy node ids,
/// in document order and deduplicated. An index selector counts the node
/// list. A path glob matches hierarchy paths with the gitignore engine. A
/// match selects that node alone, and an excluded node blocks every node below
/// it. A node reached through several parents matches when any of its paths
/// does. With neither selector every node matches. The caller decides how many
/// matches are acceptable.
pub fn select_nodes<T: VoxExt>(
    main: &VoxMain<T>,
    select: &[String],
    select_index: &[IndexRange],
) -> Result<Vec<U32Id<BVoxHierarchyNode>>> {
    let node_ids: Vec<NodeId> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .collect();

    if select.is_empty() && select_index.is_empty() {
        return Ok(node_ids);
    }

    let mut chosen: Vec<bool> = (0..node_ids.len())
        .map(|node_index| {
            select_index
                .iter()
                .any(|selector| selector.contains(node_index))
        })
        .collect();

    if !select.is_empty() {
        let patterns = GitIgnoreRegex::from_spans_ignore_inert(select)?;

        for NodePath { node_id, path, .. } in node_paths(main) {
            if !is_node_path_match(&patterns, &path) {
                continue;
            }

            let node_index = node_ids
                .iter()
                .position(|&id| id == node_id)
                .expect("node_paths walks the main's own nodes");

            chosen[node_index] = true;
        }
    }

    Ok(node_ids
        .into_iter()
        .zip(chosen)
        .filter_map(|(node_id, chosen)| chosen.then_some(node_id))
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::utilities::{
        IndexRange,
        select_nodes::{NodeId, select_nodes},
    };
    use voxcore::{VoxHierarchyNode, VoxMain};

    fn node_id(main: &mut VoxMain, name: &str, child_node_ids: Vec<NodeId>) -> NodeId {
        let node = VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids,
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap()
    }

    fn globs(patterns: &[&str]) -> Vec<String> {
        patterns.iter().map(|pattern| pattern.to_string()).collect()
    }

    fn index(index: usize) -> IndexRange {
        IndexRange::new(index, index).unwrap()
    }

    /// `house` holds `door`, which holds `knob`. `garage` also holds `door`.
    fn scene() -> (VoxMain, [NodeId; 4]) {
        let mut main = VoxMain::default();

        let knob_id = node_id(&mut main, "knob", vec![]);
        let door_id = node_id(&mut main, "door", vec![knob_id]);
        let house_id = node_id(&mut main, "house", vec![door_id]);
        let garage_id = node_id(&mut main, "garage", vec![door_id]);
        main.push_root_hierarchy_node_id(house_id).unwrap();
        main.push_root_hierarchy_node_id(garage_id).unwrap();

        (main, [knob_id, door_id, house_id, garage_id])
    }

    #[test]
    fn no_selectors_select_every_node() {
        let (main, ids) = scene();

        assert_eq!(select_nodes(&main, &[], &[]).unwrap(), ids);
    }

    #[test]
    fn select_index_counts_the_node_list() {
        let (main, [_, door_id, ..]) = scene();

        assert_eq!(select_nodes(&main, &[], &[index(1)]).unwrap(), [door_id]);
    }

    #[test]
    fn a_glob_selects_the_node_alone() {
        let (main, [_, _, house_id, _]) = scene();

        assert_eq!(
            select_nodes(&main, &globs(&["house"]), &[]).unwrap(),
            [house_id]
        );
    }

    #[test]
    fn a_node_reached_twice_counts_once() {
        let (main, [_, door_id, ..]) = scene();

        assert_eq!(
            select_nodes(&main, &globs(&["door"]), &[]).unwrap(),
            [door_id]
        );
    }

    #[test]
    fn a_node_matches_through_any_of_its_paths() {
        let (main, [_, door_id, ..]) = scene();

        // The `house` path is subtracted, but the `garage` path still matches.
        assert_eq!(
            select_nodes(&main, &globs(&["door", "!house/door"]), &[]).unwrap(),
            [door_id]
        );
    }

    #[test]
    fn an_excluded_node_blocks_what_is_below_it() {
        let (main, _) = scene();

        // `knob` is re-included after `door` is excluded, but git semantics
        // keep it out along both paths.
        assert_eq!(
            select_nodes(&main, &globs(&["!door/", "knob"]), &[]).unwrap(),
            Vec::<NodeId>::new()
        );
    }

    #[test]
    fn selectors_union_and_deduplicate() {
        let (main, [knob_id, door_id, ..]) = scene();

        assert_eq!(
            select_nodes(&main, &globs(&["knob", "door"]), &[index(0)]).unwrap(),
            [knob_id, door_id]
        );
    }
}
