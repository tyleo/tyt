/// Wraps a user-entered `name` in double quotes, escaping embedded quotes and
/// control characters, so an empty or space-bearing name stays one legible
/// token. An empty name renders as `""`.
pub(crate) fn quote_name(name: &str) -> String {
    format!("{name:?}")
}
