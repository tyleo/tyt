use std::io::Result as IOResult;

/// A codec that writes a preference type into a config file section.
///
/// The `json-codec` and `jsonc-codec` features provide `JsonCodec` and
/// `JsoncCodec` as implementations.
pub trait SerializePrefs<T> {
    /// Builds the file bytes to write back: `key` maps to an encoding of
    /// `value`, and every other top-level section of `existing` is preserved.
    /// Starts from an empty object when `existing` is `None`.
    fn serialize_prefs(&self, value: &T, key: &str, existing: Option<&[u8]>) -> IOResult<Vec<u8>>;
}
