use std::io::Result as IOResult;

/// A codec that reads a preference type from a config file section.
///
/// The `json-codec` and `jsonc-codec` features provide `JsonCodec` and
/// `JsoncCodec` as implementations.
pub trait DeserializePrefs<T> {
    /// Deserializes the `key` section from a config file's bytes. Returns
    /// `None` if the section is absent.
    fn deserialize_prefs(&self, config: &[u8], key: &str) -> IOResult<Option<T>>;
}
