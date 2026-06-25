/// An ordered list of string key/value pairs: the contents of a MagicaVoxel
/// `DICT`. The `.vox` format stores every dictionary value as a string, so both
/// halves of each pair are strings, and order is preserved as read.
///
/// The codec lifts a chunk's documented keys (a material's weight, a node's
/// name, a frame's translation) into dedicated struct fields; an `MVoxDict`
/// named `extra` on those types holds whatever keys remain, so a chunk
/// round-trips even when it carries keys this crate does not model. The decoder
/// never places a modeled key in an `extra` dictionary; constructing one there
/// is unsupported, since the typed field is the source of truth on encode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MVoxDict(pub Vec<(String, String)>);

impl MVoxDict {
    /// The value of the first pair whose key is `key`, if any.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(pair_key, _)| pair_key == key)
            .map(|(_, value)| value.as_str())
    }

    /// Whether the dictionary holds no pairs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<(String, String)>> for MVoxDict {
    fn from(pairs: Vec<(String, String)>) -> Self {
        MVoxDict(pairs)
    }
}
