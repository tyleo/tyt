use crate::{Error, Result, parse_id_selector};
use branded_id::U32Id;
use clap::Args;
use voxcore::{BVoxObject, VoxExt, VoxMain};
use voxsmith::utilities::{IdSelector, select_objects};

/// The `--select` / `--select-index` object selectors, shared by every command
/// that narrows its work to some of a document's objects.
#[derive(Clone, Debug, Args)]
pub struct ObjectSelection {
    /// Chooses objects by gitignore-style hierarchy-path pattern, matched as
    /// `node list` matches node paths, so a node path selects its subtree.
    /// Repeatable; unions with `--select-index`.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// Chooses objects by id: an integer, an inclusive `a-b` range, or `*` for
    /// every object. Repeatable; unions with `--select`.
    #[arg(value_name = "select-index", long, value_parser = parse_id_selector::<BVoxObject>)]
    select_index: Vec<IdSelector<BVoxObject>>,
}

impl ObjectSelection {
    /// Whether any selector was given. With none, every object is selected.
    pub fn has_selectors(&self) -> bool {
        !self.select.is_empty() || !self.select_index.is_empty()
    }

    /// The ids of the objects the selectors match in `main`, in document
    /// order. An id the document lacks errors, and a selection that matches
    /// nothing is a usage error, so a stray glob is caught.
    pub fn resolve<T: VoxExt>(&self, main: &VoxMain<T>) -> Result<Vec<U32Id<BVoxObject>>> {
        let object_ids = select_objects(main, &self.select, &self.select_index)?;

        if self.has_selectors() && object_ids.is_empty() {
            return Err(Error::usage(
                "no object matched the selection; check --select and --select-index",
            ));
        }

        Ok(object_ids)
    }
}

#[cfg(test)]
mod tests {
    use crate::try_parse_object_selection;
    use voxcore::VoxMain;

    #[test]
    fn no_flags_give_no_selectors() {
        assert!(!try_parse_object_selection(&[]).unwrap().has_selectors());
        assert!(
            try_parse_object_selection(&["--select", "door"])
                .unwrap()
                .has_selectors()
        );
        assert!(
            try_parse_object_selection(&["--select-index", "0-2"])
                .unwrap()
                .has_selectors()
        );
    }

    #[test]
    fn a_bad_index_is_a_parse_error() {
        assert!(try_parse_object_selection(&["--select-index", "x"]).is_err());
        assert!(try_parse_object_selection(&["--select-index", "5-2"]).is_err());
    }

    #[test]
    fn a_selector_matching_nothing_is_an_error_but_no_selector_is_not() {
        // An empty document: nothing to match.
        let main: VoxMain = VoxMain::default();

        assert!(
            try_parse_object_selection(&["--select", "door"])
                .unwrap()
                .resolve(&main)
                .is_err()
        );
        assert!(
            try_parse_object_selection(&["--select-index", "3"])
                .unwrap()
                .resolve(&main)
                .is_err()
        );
        assert!(
            try_parse_object_selection(&[])
                .unwrap()
                .resolve(&main)
                .unwrap()
                .is_empty()
        );
    }
}
