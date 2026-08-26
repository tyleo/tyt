use crate::DirPrefs;

/// A [`DirPrefs`] whose prefs may be absent.
pub type OptionalDirPrefs<T> = DirPrefs<Option<T>>;
