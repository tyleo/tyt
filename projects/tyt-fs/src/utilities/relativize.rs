use std::path::{Component, Path, PathBuf};

/// Re-expresses `target` as a path relative to `base`.
///
/// Both paths are first normalized lexically (without touching the filesystem):
/// `.` components are dropped and `..` components are resolved against earlier
/// components. The relative path is then computed from the components the two
/// paths do not share. If the paths do not share a common root (and so cannot
/// be relativized), the normalized `target` is returned unchanged.
pub fn relativize(base: &Path, target: &Path) -> PathBuf {
    let base = normalize(base);
    let target = normalize(target);

    let base_components: Vec<Component> = base.components().collect();
    let target_components: Vec<Component> = target.components().collect();

    let common = base_components
        .iter()
        .zip(target_components.iter())
        .take_while(|(b, t)| b == t)
        .count();

    let base_rest = &base_components[common..];
    let target_rest = &target_components[common..];

    // A leftover root or prefix in `base` means the two paths do not share a
    // root, so no `..`-prefixed relative path can reach `target`.
    if base_rest
        .iter()
        .any(|c| matches!(c, Component::Prefix(_) | Component::RootDir))
    {
        return target;
    }

    let mut result = PathBuf::new();
    for _ in base_rest {
        result.push("..");
    }
    for component in target_rest {
        result.push(component.as_os_str());
    }

    if result.as_os_str().is_empty() {
        result.push(".");
    }
    result
}

/// Resolves `.` and `..` components lexically without consulting the
/// filesystem. `..` pops a preceding normal component, is dropped at a root,
/// and is otherwise retained (for relative paths that escape their start).
fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match result.components().next_back() {
                Some(Component::Normal(_)) => {
                    result.pop();
                }
                Some(Component::RootDir | Component::Prefix(_)) => {}
                _ => result.push(".."),
            },
            other => result.push(other.as_os_str()),
        }
    }
    result
}
