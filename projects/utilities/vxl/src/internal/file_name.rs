use std::path::Path;

/// The file name of `path` for a report heading, or its full path when it has
/// none.
pub(crate) fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string())
}
