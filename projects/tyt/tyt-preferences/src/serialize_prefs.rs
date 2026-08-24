use std::io::Result as IOResult;

/// Abstracts JSON serialization for preference types.
pub trait SerializePrefs {
    /// Builds the pretty-printed file bytes to write back: `key` maps to a
    /// JSON encoding of `self`, and every other top-level section of
    /// `existing` is preserved. Starts from an empty object when `existing`
    /// is `None`.
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>>;
}
