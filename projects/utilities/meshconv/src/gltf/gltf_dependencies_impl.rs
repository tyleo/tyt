use crate::{DependenciesImpl, gltf::GltfDependencies};
use gltf_meshdoc::DependenciesImpl as GltfDependenciesImpl;

impl GltfDependencies for DependenciesImpl {
    type Gltf = GltfDependenciesImpl;

    fn gltf(&self) -> &Self::Gltf {
        &GltfDependenciesImpl
    }
}
