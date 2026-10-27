use crate::{
    CreateDirAll, CreateDirLink,
    commands::{AGENTS_DIR, CLAUDE_DIR},
};
use std::{
    io::Result as IOResult,
    path::{Path, PathBuf},
};

/// The relative target `.agents` survives a move or a fresh clone of `root`.
pub fn link_claude(
    dependencies: &(impl CreateDirAll + CreateDirLink),
    root: &Path,
) -> IOResult<PathBuf> {
    let link = root.join(CLAUDE_DIR);

    dependencies.create_dir_all(&root.join(AGENTS_DIR))?;

    dependencies.create_dir_link(Path::new(AGENTS_DIR), &link)?;

    Ok(link)
}
