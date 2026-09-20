use crate::{
    Dimension, Domain, Scalar,
    checker::{CheckResult, CheckedNode},
};

/// A subtree of bare literals and the type-keeping operations over them,
/// awaiting the numeric type its context fixes.
pub(crate) struct Pending {
    /// What the subtree has one entry per.
    pub(crate) domain: Domain,

    /// The subtree's vec width.
    pub(crate) dimension: Dimension,

    /// Builds the checked subtree once the type is known.
    pub(crate) build: Box<dyn FnOnce(Scalar) -> CheckResult<CheckedNode>>,
}

impl Pending {
    /// Settles the subtree to the given type.
    pub(crate) fn resolve(self, scalar: Scalar) -> CheckResult<CheckedNode> {
        (self.build)(scalar)
    }
}
