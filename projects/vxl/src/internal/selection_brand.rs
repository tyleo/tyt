use voxcore::{BVoxHierarchyNode, BVoxObject};

/// The kind of entry a [`RequiredSelection`](crate::RequiredSelection)
/// selects, carrying the help its two flags print.
pub trait SelectionBrand: Send + Sync + 'static {
    /// The `--select` help.
    const SELECT_HELP: &'static str;

    /// The `--select-index` help.
    const SELECT_INDEX_HELP: &'static str;
}

impl SelectionBrand for BVoxObject {
    const SELECT_HELP: &'static str = "Chooses objects by gitignore-style hierarchy-path \
        pattern, matched as `node list` matches node paths, so a node path selects its \
        subtree. Repeatable; unions with `--select-index`";

    const SELECT_INDEX_HELP: &'static str = "Chooses objects by id: an integer, an inclusive \
        `a-b` range, or `*` for every object. Repeatable; unions with `--select`";
}

impl SelectionBrand for BVoxHierarchyNode {
    const SELECT_HELP: &'static str = "Chooses nodes by gitignore-style hierarchy-path pattern, \
        matched as `node list` matches node paths; a matched path selects that node alone. \
        Repeatable; unions with `--select-index`";

    const SELECT_INDEX_HELP: &'static str = "Chooses nodes by id: an integer, an inclusive \
        `a-b` range, or `*` for every node. Repeatable; unions with `--select`";
}
