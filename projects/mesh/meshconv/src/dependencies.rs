#[cfg(feature = "gltf")]
use crate::gltf::GltfDependencies;

/// The codec dependencies the read and write functions take: each enabled
/// format's, through that format's dependencies trait. A type implementing
/// every enabled format's trait implements this one. Each format keeps its
/// own associated type, so two formats' same-named traits never meet in one
/// bound. A type holding the dependencies in another value implements
/// [`ForwardDependencies`](crate::ForwardDependencies) instead.
pub trait Dependencies: GltfDependencies {}

impl<D: GltfDependencies> Dependencies for D {}

/// Stands in for the glTF dependencies without the `gltf` feature. Every
/// type implements it.
#[cfg(not(feature = "gltf"))]
pub trait GltfDependencies {}

#[cfg(not(feature = "gltf"))]
impl<D> GltfDependencies for D {}
