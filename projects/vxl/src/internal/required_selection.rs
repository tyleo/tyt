use crate::{Error, Result, SelectionBrand, parse_id_selector};
use branded_id::U32Id;
use clap::Args;
use std::fmt;
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxExt, VoxMain};
use voxsmith::utilities::{IdSelector, select_nodes, select_objects};

/// The `--select` / `--select-index` selectors of an edit command, which
/// requires at least one. `TBrand` is the kind of entry the command selects
/// and supplies the flags' help.
#[derive(Args)]
#[group(required = true, multiple = true)]
pub struct RequiredSelection<TBrand: SelectionBrand> {
    #[arg(value_name = "select", long, help = TBrand::SELECT_HELP)]
    select: Vec<String>,

    #[arg(
        value_name = "select-index",
        long,
        help = TBrand::SELECT_INDEX_HELP,
        value_parser = parse_id_selector::<TBrand>
    )]
    select_index: Vec<IdSelector<TBrand>>,
}

impl RequiredSelection<BVoxObject> {
    /// The ids of the objects the selectors match in `main`, in document
    /// order. Errors when they match nothing.
    pub fn resolve_objects<T: VoxExt>(&self, main: &VoxMain<T>) -> Result<Vec<U32Id<BVoxObject>>> {
        let object_ids = select_objects(main, &self.select, &self.select_index)?;

        if object_ids.is_empty() {
            return Err(Error::usage(
                "no object matched the selection; check --select and --select-index",
            ));
        }

        Ok(object_ids)
    }

    /// The one object the selectors match in `main`. Errors unless they match
    /// exactly one.
    pub fn resolve_one_object<T: VoxExt>(&self, main: &VoxMain<T>) -> Result<U32Id<BVoxObject>> {
        let object_ids = self.resolve_objects(main)?;

        let [object_id] = object_ids[..] else {
            return Err(Error::usage(format!(
                "the selection matched {} objects but this command needs exactly one; check \
                 --select and --select-index",
                object_ids.len()
            )));
        };

        Ok(object_id)
    }
}

impl RequiredSelection<BVoxHierarchyNode> {
    /// The ids of the nodes the selectors match in `main`, in document order.
    /// Errors when they match nothing.
    pub fn resolve_nodes<T: VoxExt>(
        &self,
        main: &VoxMain<T>,
    ) -> Result<Vec<U32Id<BVoxHierarchyNode>>> {
        let node_ids = select_nodes(main, &self.select, &self.select_index)?;

        if node_ids.is_empty() {
            return Err(Error::usage(
                "no node matched the selection; check --select and --select-index",
            ));
        }

        Ok(node_ids)
    }

    /// The one node the selectors match in `main`. Errors unless they match
    /// exactly one.
    pub fn resolve_one_node<T: VoxExt>(
        &self,
        main: &VoxMain<T>,
    ) -> Result<U32Id<BVoxHierarchyNode>> {
        let node_ids = self.resolve_nodes(main)?;

        let [node_id] = node_ids[..] else {
            return Err(Error::usage(format!(
                "the selection matched {} nodes but this command needs exactly one; check \
                 --select and --select-index",
                node_ids.len()
            )));
        };

        Ok(node_id)
    }
}

impl<TBrand: SelectionBrand> Clone for RequiredSelection<TBrand> {
    fn clone(&self) -> Self {
        Self {
            select: self.select.clone(),
            select_index: self.select_index.clone(),
        }
    }
}

impl<TBrand: SelectionBrand> fmt::Debug for RequiredSelection<TBrand> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RequiredSelection")
            .field("select", &self.select)
            .field("select_index", &self.select_index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::RequiredSelection;
    use clap::Parser;
    use ty_math::TyVector3U32;
    use voxcore::{BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject};

    #[derive(Debug, Parser)]
    struct ObjectCli {
        #[command(flatten)]
        selection: RequiredSelection<BVoxObject>,
    }

    #[derive(Debug, Parser)]
    struct NodeCli {
        #[command(flatten)]
        selection: RequiredSelection<BVoxHierarchyNode>,
    }

    fn argv<'a>(args: &[&'a str]) -> Vec<&'a str> {
        let mut argv = vec!["cli"];

        argv.extend_from_slice(args);

        argv
    }

    fn object_selection(args: &[&str]) -> RequiredSelection<BVoxObject> {
        ObjectCli::try_parse_from(argv(args)).unwrap().selection
    }

    fn node_selection(args: &[&str]) -> RequiredSelection<BVoxHierarchyNode> {
        NodeCli::try_parse_from(argv(args)).unwrap().selection
    }

    #[test]
    fn a_selector_is_required() {
        assert!(ObjectCli::try_parse_from(["cli"]).is_err());
        assert!(ObjectCli::try_parse_from(["cli", "--select", "a", "--select-index", "0"]).is_ok());
        assert!(ObjectCli::try_parse_from(["cli", "--select-index", "*"]).is_ok());
    }

    #[test]
    fn a_selection_matching_nothing_is_an_error() {
        let mut main: VoxMain = VoxMain::default();

        let node = VoxHierarchyNode {
            name: "door".to_owned(),
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap();

        assert!(
            object_selection(&["--select", "door"])
                .resolve_objects(&main)
                .is_err()
        );
        assert!(
            node_selection(&["--select", "wall"])
                .resolve_nodes(&main)
                .is_err()
        );
        assert_eq!(
            node_selection(&["--select", "door"])
                .resolve_nodes(&main)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn resolve_one_object_needs_exactly_one_match() {
        let mut main: VoxMain = VoxMain::default();

        for name in ["a", "b"] {
            let object = VoxObject::new(name.to_owned(), TyVector3U32::splat(1)).unwrap();

            main.retain_object(object).unwrap();
        }

        assert!(
            object_selection(&["--select", "a"])
                .resolve_one_object(&main)
                .is_ok()
        );
        assert!(
            object_selection(&["--select-index", "0-1"])
                .resolve_one_object(&main)
                .is_err()
        );
    }

    #[test]
    fn resolve_one_node_needs_exactly_one_match() {
        let mut main: VoxMain = VoxMain::default();

        for name in ["a", "b"] {
            let node = VoxHierarchyNode {
                name: name.to_owned(),
                ..Default::default()
            };

            main.retain_hierarchy_node(node).unwrap();
        }

        assert!(
            node_selection(&["--select", "b"])
                .resolve_one_node(&main)
                .is_ok()
        );
        assert!(
            node_selection(&["--select-index", "*"])
                .resolve_one_node(&main)
                .is_err()
        );
    }
}
