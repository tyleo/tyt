use crate::{
    CheckFailure, Type,
    checker::{CheckResult, Checked, CheckedNode, check_node},
    parser::SyntaxNode,
};
use std::collections::HashMap;

/// Checks a whole expression, erroring where its bare literals never met a
/// type.
pub(crate) fn check_root(
    node: &SyntaxNode,
    scope: &HashMap<String, Type>,
) -> CheckResult<CheckedNode> {
    match check_node(node, scope)? {
        Checked::Pending(_) => Err(CheckFailure::UntypedLiteral),
        Checked::Typed(node) => Ok(node),
    }
}
