use crate::{CliValue, Result, commands::push_unique};
use voxsmith::operations::mesh::ArrayDomain;

/// Pushes `domain` onto a stream list. A domain listed twice errors with a
/// message `lists` opens.
pub(crate) fn push_uv_stream(
    streams: &mut Vec<ArrayDomain>,
    domain: ArrayDomain,
    lists: impl FnOnce() -> String,
) -> Result<()> {
    push_unique(
        streams,
        domain,
        |domain| *domain,
        |domain| format!("{} `{}` twice", lists(), domain.name()),
    )
}
