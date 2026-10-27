use std::collections::BTreeMap;

/// A written document as bytes: the primary file and the loose files that go
/// beside it, keyed by the relative URI the primary references them by.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GltfBytes {
    /// The `.gltf` or `.glb` bytes.
    pub primary: Vec<u8>,

    /// The loose files, keyed by relative URI.
    pub loose_files: BTreeMap<String, Vec<u8>>,
}
