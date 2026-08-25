use std::io::Result as IOResult;

/// Abstracts JSON deserialization for preference types.
///
/// The `impl-json` feature provides `deserialize_prefs_json` as a ready-made
/// implementation body.
pub trait DeserializePrefs: Sized {
    /// Deserializes the `key` section from a config file's JSON bytes.
    fn deserialize_prefs(config_json: &[u8], key: &str) -> IOResult<Option<Self>>;
}
