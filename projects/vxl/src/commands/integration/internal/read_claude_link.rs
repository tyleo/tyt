use crate::{
    PathKind, ReadPathKind,
    commands::{AGENTS_DIR, CLAUDE_DIR, ClaudeLink},
};
use std::{
    io::Result as IOResult,
    path::{Component, Path, PathBuf},
};

/// What sits at `.claude` under `root`. A symlink counts as linked when its
/// target points to `root`'s `.agents`, relatively or absolutely.
pub fn read_claude_link(dependencies: &impl ReadPathKind, root: &Path) -> IOResult<ClaudeLink> {
    let claude_link = match dependencies.read_path_kind(&root.join(CLAUDE_DIR))? {
        PathKind::Missing => ClaudeLink::Missing,

        PathKind::File => ClaudeLink::File,

        PathKind::Directory => ClaudeLink::Directory,

        PathKind::Symlink { target } => {
            if normalize(&root.join(&target)) == normalize(&root.join(AGENTS_DIR)) {
                ClaudeLink::Linked
            } else {
                ClaudeLink::LinkedElsewhere { target }
            }
        }
    };

    Ok(claude_link)
}

/// `path` with its `.` and `..` components resolved lexically.
fn normalize(path: &Path) -> PathBuf {
    let mut normal = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}

            Component::ParentDir => {
                normal.pop();
            }

            component => normal.push(component),
        }
    }

    normal
}
