use std::path::PathBuf;

/// A directory paired with prefs parsed from its config file.
#[derive(Clone, Debug)]
pub struct DirPrefs<T> {
    /// Directory the config file lives in.
    pub dir: PathBuf,

    /// Prefs parsed from the config file.
    pub prefs: T,
}
