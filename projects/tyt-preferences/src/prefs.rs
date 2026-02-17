/// Preferences loaded from `.tytconfig` files.
#[derive(Clone, Debug, Default)]
pub struct Prefs<T> {
    /// Preferences from `~/.tytconfig`.
    pub user: Option<T>,
    /// Preferences from `<git-root>/.tytconfig`.
    pub git_root: Option<T>,
}
