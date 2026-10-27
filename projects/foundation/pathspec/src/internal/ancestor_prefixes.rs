/// The proper ancestor prefixes of `path`, shallowest first. `a/b/c` gives `a`
/// then `a/b`.
pub fn ancestor_prefixes(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/')
        .map(move |(index, _)| &path[..index])
}
