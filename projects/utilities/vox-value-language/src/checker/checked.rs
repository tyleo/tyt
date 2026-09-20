use crate::{
    Dimension, Domain, Scalar,
    checker::{CheckResult, CheckedNode, Pending},
};

/// A checked subtree: typed, or pending the type its context fixes.
pub(crate) enum Checked {
    Pending(Pending),
    Typed(CheckedNode),
}

impl Checked {
    /// The vec width.
    pub(crate) fn dimension(&self) -> Dimension {
        match self {
            Checked::Pending(pending) => pending.dimension,
            Checked::Typed(node) => node.output.dimension,
        }
    }

    /// What the subtree has one entry per.
    pub(crate) fn domain(&self) -> Domain {
        match self {
            Checked::Pending(pending) => pending.domain,
            Checked::Typed(node) => node.output.domain,
        }
    }

    /// The pending subtree, if the type is still open.
    pub(crate) fn into_pending(self) -> Option<Pending> {
        match self {
            Checked::Pending(pending) => Some(pending),
            Checked::Typed(_) => None,
        }
    }

    /// The typed node, if the type is settled.
    pub(crate) fn into_typed(self) -> Option<CheckedNode> {
        match self {
            Checked::Pending(_) => None,
            Checked::Typed(node) => Some(node),
        }
    }

    /// The node. A pending subtree settles to the given type; a typed one
    /// comes back unchanged.
    pub(crate) fn resolve(self, scalar: Scalar) -> CheckResult<CheckedNode> {
        match self {
            Checked::Pending(pending) => pending.resolve(scalar),
            Checked::Typed(node) => Ok(node),
        }
    }

    /// The component type, none while pending.
    pub(crate) fn scalar(&self) -> Option<Scalar> {
        match self {
            Checked::Pending(_) => None,
            Checked::Typed(node) => Some(node.output.scalar),
        }
    }
}
