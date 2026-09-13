use crate::ForwardDependencies;
use gltf_meshdoc::{DecodeBase64, EncodeBase64};

/// The glTF bridge's dependencies a caller supplies.
pub trait GltfDependencies {
    /// The glTF bridge's dependencies.
    type Gltf: DecodeBase64 + EncodeBase64;

    /// The glTF bridge's dependencies.
    fn gltf(&self) -> &Self::Gltf;
}

/// Forwards to the target's.
impl<T: ForwardDependencies<Target: GltfDependencies>> GltfDependencies for T {
    type Gltf = <T::Target as GltfDependencies>::Gltf;

    fn gltf(&self) -> &Self::Gltf {
        self.target().gltf()
    }
}
