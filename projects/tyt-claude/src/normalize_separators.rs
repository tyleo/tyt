/// Rewrites path separators to the platform-native form. On Windows, `/` is
/// converted to `\`. On Unix this is a no-op (backslash is a legal filename
/// character).
pub(crate) fn normalize_separators(s: &str) -> String {
    if cfg!(windows) {
        s.replace('/', "\\")
    } else {
        s.to_string()
    }
}
