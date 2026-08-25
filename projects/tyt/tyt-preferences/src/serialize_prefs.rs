use std::io::Result as IOResult;

/// Abstracts config serialization for preference types.
///
/// The `impl-json` and `impl-jsonc` features provide `serialize_prefs_json`
/// and `serialize_prefs_jsonc` as ready-made implementation bodies.
pub trait SerializePrefs {
    /// Builds the file bytes to write back: `key` maps to an encoding of
    /// `self`, and every other top-level section of `existing` is preserved.
    /// Starts from an empty object when `existing` is `None`.
    fn serialize_prefs(&self, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>>;
}
