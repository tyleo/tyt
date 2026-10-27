use crate::{Error, Result};
use std::path::{Component, Path, PathBuf};

/// The path a document in the absolute `document_dir` holds for the font
/// `name`'s file at `path`. A relative `path` resolves against the absolute
/// `base` and lands relative to `document_dir`. An absolute one stays as it is.
pub fn document_font_path(
    name: &str,
    path: &Path,
    base: &Path,
    document_dir: &Path,
) -> Result<String> {
    let document_path = if path.is_absolute() {
        path.to_str().map(str::to_owned)
    } else {
        relative_path(&base.join(path), document_dir)
    };
    document_path.ok_or_else(|| {
        Error::usage(format!(
            "the font `{name}` takes the path `{}`, which is not UTF-8",
            path.display()
        ))
    })
}

/// The absolute `path` as seen from the absolute `directory`, with `/` between
/// its components, or `None` when it is not UTF-8. A path on another root
/// stays absolute.
fn relative_path(path: &Path, directory: &Path) -> Option<String> {
    let path = lexical_components(path);
    let directory = lexical_components(directory);
    let shared = path
        .iter()
        .zip(&directory)
        .take_while(|(path, directory)| path == directory)
        .count();

    if shared == 0 {
        return path.iter().collect::<PathBuf>().to_str().map(str::to_owned);
    }

    let parents = directory[shared..].iter().map(|_| Some(".."));
    let rest = path[shared..]
        .iter()
        .map(|component| component.as_os_str().to_str());
    Some(parents.chain(rest).collect::<Option<Vec<_>>>()?.join("/"))
}

/// The components of the absolute `path` with each `.` dropped and each `..`
/// taking away the component before it.
fn lexical_components(path: &Path) -> Vec<Component<'_>> {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}

            Component::ParentDir => {
                if matches!(components.last(), Some(Component::Normal(_))) {
                    components.pop();
                }
            }

            _ => components.push(component),
        }
    }
    components
}
