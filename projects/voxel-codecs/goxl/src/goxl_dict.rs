/// An ordered list of key/value pairs: the trailing `DICT` of a `.gox` chunk.
/// Keys are UTF-8 strings; values are raw bytes, since the format stores each
/// value as a typed binary blob.
///
/// The codec lifts a chunk's documented keys (a layer's name, a material's
/// color, a camera's distance) into dedicated struct fields; a `GoxlDict` named
/// `extra` on those types holds whatever keys remain, so a chunk round-trips
/// even when it carries keys this crate does not model. The decoder never places
/// a modeled key in an `extra` dictionary; the typed field is the source of
/// truth on encode.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GoxlDict(pub Vec<(String, Vec<u8>)>);

impl GoxlDict {
    /// The value of the first pair whose key is `key`, if any.
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.0
            .iter()
            .find(|(pair_key, _)| pair_key == key)
            .map(|(_, value)| value.as_slice())
    }

    /// Whether the dictionary holds no pairs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<(String, Vec<u8>)>> for GoxlDict {
    fn from(pairs: Vec<(String, Vec<u8>)>) -> Self {
        GoxlDict(pairs)
    }
}
