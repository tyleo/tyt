use crate::{Error, Inflate, Result};
use ty_zip::deflated_member;

/// Inflates the single deflate member of a `.sdfjz` archive back to its `.sdfj`
/// bytes through `dependencies`, the inverse of the archive
/// [`wrap_sdfjz`](crate::wrap_sdfjz()) writes. The deflate stream ends itself,
/// so any trailing zip central-directory bytes are ignored.
pub fn unwrap_sdfjz<D: Inflate>(dependencies: &D, bytes: &[u8]) -> Result<Vec<u8>> {
    let compressed = deflated_member(bytes)
        .ok_or_else(|| Error::Invalid("malformed .sdfjz archive".to_owned()))?;
    dependencies
        .inflate(compressed)
        .map_err(|reason| Error::Invalid(format!("malformed .sdfjz deflate stream: {reason}")))
}
