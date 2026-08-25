use std::io::Result as IOResult;

/// Abstracts config deserialization for preference types.
///
/// The `impl-json` and `impl-jsonc` features provide `deserialize_prefs_json`
/// and `deserialize_prefs_jsonc` as ready-made implementation bodies.
pub trait DeserializePrefs: Sized {
    /// Deserializes the `key` section from a config file's bytes.
    fn deserialize_prefs(config: &[u8], key: &str) -> IOResult<Option<Self>>;
}
