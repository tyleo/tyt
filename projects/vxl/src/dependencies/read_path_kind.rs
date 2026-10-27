use crate::PathKind;
use std::{io::Result as IOResult, path::Path};

/// Reads what sits at a path.
pub trait ReadPathKind {
    /// What sits at `path`. A symlink at `path` is reported, not followed.
    fn read_path_kind(&self, path: &Path) -> IOResult<PathKind>;
}
