use crate::{Dependencies, Result};

/// Returns, per path in `paths`, whether the gitignore-style `patterns` match
/// it. A path counts as a directory when another path nests under it, so a
/// directory-only pattern can match it. The returned flags line up with
/// `paths`.
pub fn match_hierarchy_paths(
    dependencies: &impl Dependencies,
    patterns: &[String],
    paths: &[&str],
) -> Result<Vec<bool>> {
    let candidates: Vec<(&str, bool)> = paths
        .iter()
        .map(|&path| (path, has_child(path, paths)))
        .collect();

    let pattern_refs: Vec<&str> = patterns.iter().map(String::as_str).collect();

    dependencies.match_paths(&pattern_refs, &candidates)
}

/// Whether any path in `paths` nests strictly under `path`.
fn has_child(path: &str, paths: &[&str]) -> bool {
    let prefix = format!("{path}/");
    paths.iter().any(|candidate| candidate.starts_with(&prefix))
}
