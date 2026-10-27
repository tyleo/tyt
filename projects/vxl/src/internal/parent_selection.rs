use crate::{Error, Result, parse_id_selector};
use branded_id::U32Id;
use clap::Args;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};
use voxsmith::utilities::{IdSelector, select_nodes};

/// The `--select-parent` / `--select-parent-index` selectors, which pick the
/// parent end of an edge.
#[derive(Clone, Debug, Args)]
pub struct ParentSelection {
    /// Chooses the parent node by gitignore-style hierarchy-path pattern,
    /// matched as `node list` matches node paths; a matched path selects that
    /// node alone. Repeatable; unions with `--select-parent-index`.
    #[arg(value_name = "select-parent", long)]
    select_parent: Vec<String>,

    /// Chooses the parent node by id: an integer, an inclusive `a-b` range, or
    /// `*` for every node. Repeatable; unions with `--select-parent`.
    #[arg(
        value_name = "select-parent-index",
        long,
        value_parser = parse_id_selector::<BVoxHierarchyNode>
    )]
    select_parent_index: Vec<IdSelector<BVoxHierarchyNode>>,
}

impl ParentSelection {
    /// The parent node the selectors match in `main`. Returns `None` for the
    /// root list when no selector was given, and errors unless the selectors
    /// match exactly one node.
    pub fn resolve<T: VoxExt>(
        &self,
        main: &VoxMain<T>,
    ) -> Result<Option<U32Id<BVoxHierarchyNode>>> {
        if self.select_parent.is_empty() && self.select_parent_index.is_empty() {
            return Ok(None);
        }

        let node_ids = select_nodes(main, &self.select_parent, &self.select_parent_index)?;

        let [node_id] = node_ids[..] else {
            return Err(Error::usage(format!(
                "the parent selection matched {} nodes but must match exactly one; check \
                 --select-parent and --select-parent-index",
                node_ids.len()
            )));
        };

        Ok(Some(node_id))
    }

    /// The parent node the selectors match in `main`, for a command that needs
    /// one. Errors unless a selector was given and the selectors match exactly
    /// one node.
    pub fn resolve_required<T: VoxExt>(
        &self,
        main: &VoxMain<T>,
    ) -> Result<U32Id<BVoxHierarchyNode>> {
        let Some(node_id) = self.resolve(main)? else {
            return Err(Error::usage(
                "this command needs a parent node; pass --select-parent or --select-parent-index",
            ));
        };

        Ok(node_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::ParentSelection;
    use clap::Parser;
    use voxcore::{VoxHierarchyNode, VoxMain};

    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        parent: ParentSelection,
    }

    fn parent(args: &[&str]) -> ParentSelection {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().parent
    }

    /// Two root nodes, both named `door`.
    fn two_doors() -> VoxMain {
        let mut main = VoxMain::default();

        for _ in 0..2 {
            let node = VoxHierarchyNode {
                name: "door".to_owned(),
                ..Default::default()
            };

            let node_id = main.retain_hierarchy_node(node).unwrap();

            main.push_root_hierarchy_node_id(node_id).unwrap();
        }

        main
    }

    #[test]
    fn no_selector_targets_the_root_list() {
        assert_eq!(parent(&[]).resolve(&two_doors()).unwrap(), None);
    }

    #[test]
    fn a_required_parent_needs_a_selector() {
        let main = two_doors();

        assert!(parent(&[]).resolve_required(&main).is_err());
        assert!(
            parent(&["--select-parent-index", "0"])
                .resolve_required(&main)
                .is_ok()
        );
    }

    #[test]
    fn the_selectors_must_match_exactly_one_node() {
        let main = two_doors();

        assert!(parent(&["--select-parent", "door"]).resolve(&main).is_err());
        assert!(parent(&["--select-parent", "wall"]).resolve(&main).is_err());
        assert!(
            parent(&["--select-parent-index", "1"])
                .resolve(&main)
                .unwrap()
                .is_some()
        );
    }
}
