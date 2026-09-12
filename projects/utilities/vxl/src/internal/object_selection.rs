use crate::{Error, Result, parse_index_range};
use branded_id::U32Id;
use clap::Args;
use voxcore::{BVoxObject, VoxExt, VoxMain};
use voxsmith::utilities::{IndexRange, select_objects};

/// The `--select` / `--select-index` object selectors, shared by every command
/// that narrows its work to some of a document's objects.
#[derive(Clone, Debug, Args)]
pub struct ObjectSelection {
    /// Choose objects by hierarchy-path glob, matched as `hierarchy show`
    /// matches node paths, so a node path selects its subtree. Repeatable;
    /// unions with `--select-index`.
    #[arg(value_name = "select", long)]
    select: Vec<String>,

    /// Choose objects by index, an integer or an `a-b` range. Repeatable;
    /// unions with `--select`.
    #[arg(value_name = "select-index", long, value_parser = parse_index_range)]
    select_index: Vec<IndexRange>,
}

impl ObjectSelection {
    /// Whether any selector was given. With none, every object is selected.
    pub fn has_selectors(&self) -> bool {
        !self.select.is_empty() || !self.select_index.is_empty()
    }

    /// The ids of the objects the selectors match in `main`, in document
    /// order. A selector that matches nothing is a usage error, so a stray
    /// glob or index is caught.
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
    use crate::ObjectSelection;
    use clap::Parser;
    use voxcore::VoxMain;

    /// A command carrying only the selectors.
    #[derive(Debug, Parser)]
    struct Cli {
        #[command(flatten)]
        selection: ObjectSelection,
    }

    /// The selectors parsed from `args`.
    fn selection(args: &[&str]) -> ObjectSelection {
        let mut argv = vec!["cli"];
        argv.extend_from_slice(args);
        Cli::try_parse_from(argv).unwrap().selection
    }

    #[test]
    fn no_flags_give_no_selectors() {
        assert!(!selection(&[]).has_selectors());
        assert!(selection(&["--select", "door"]).has_selectors());
        assert!(selection(&["--select-index", "0-2"]).has_selectors());
    }

    #[test]
    fn a_bad_index_is_a_parse_error() {
        assert!(Cli::try_parse_from(["cli", "--select-index", "x"]).is_err());
        assert!(Cli::try_parse_from(["cli", "--select-index", "5-2"]).is_err());
    }

    #[test]
    fn a_selector_matching_nothing_is_an_error_but_no_selector_is_not() {
        // An empty document: nothing to match.
        let main: VoxMain = VoxMain::default();

        assert!(selection(&["--select", "door"]).resolve(&main).is_err());
        assert!(selection(&["--select-index", "3"]).resolve(&main).is_err());
        assert!(selection(&[]).resolve(&main).unwrap().is_empty());
    }
}
