use crate::GltfExt;
use meshdoc::MeshMain;

/// A [`MeshMain`] carrying its glTF provenance as its ext, the state the
/// converters exchange.
pub type GltfMeshMain = MeshMain<GltfExt>;
