use gltf::json::Root;
use std::collections::BTreeMap;

/// A glTF document as its parts. Either container reads into and writes
/// from this shape.
#[derive(Clone, Debug, Default)]
pub struct GltfFile {
    /// The JSON root.
    pub root: Root,

    /// The bytes of the first buffer when it has no URI: a GLB's `BIN`
    /// chunk, or the buffer a `.gltf` embeds as a data URI.
    pub blob: Option<Vec<u8>>,

    /// The files beside the document, keyed by the relative URI the root
    /// references them by.
    pub loose_files: BTreeMap<String, Vec<u8>>,
}
