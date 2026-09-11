#[cfg(feature = "goxl")]
use crate::goxl::GoxlDependencies;
#[cfg(feature = "qbcl")]
use crate::qbcl::QbclDependencies;
#[cfg(feature = "vmax")]
use crate::vmax::VMaxDependencies;
#[cfg(feature = "voxj")]
use crate::voxj::VoxjDependencies;

/// The codec dependencies the read and write functions take: each enabled
/// format's, through that format's dependencies trait. A type implementing
/// every enabled format's trait implements this one. The formats' codec crates
/// share trait names such as `DecodePng` with differing signatures, so each
/// format keeps its own associated type instead of one bound over all of them.
/// MagicaVoxel's codec needs none. A type holding the dependencies in another
/// value implements [`ForwardDependencies`](crate::ForwardDependencies)
/// instead.
pub trait Dependencies:
    GoxlDependencies + QbclDependencies + VMaxDependencies + VoxjDependencies
{
}

impl<D: GoxlDependencies + QbclDependencies + VMaxDependencies + VoxjDependencies> Dependencies
    for D
{
}

/// Stands in for the Goxel dependencies without the `goxl` feature. Every
/// type implements it.
#[cfg(not(feature = "goxl"))]
pub trait GoxlDependencies {}

#[cfg(not(feature = "goxl"))]
impl<D> GoxlDependencies for D {}

/// Stands in for the Qubicle dependencies without the `qbcl` feature. Every
/// type implements it.
#[cfg(not(feature = "qbcl"))]
pub trait QbclDependencies {}

#[cfg(not(feature = "qbcl"))]
impl<D> QbclDependencies for D {}

/// Stands in for the Voxel Max dependencies without the `vmax` feature.
/// Every type implements it.
#[cfg(not(feature = "vmax"))]
pub trait VMaxDependencies {}

#[cfg(not(feature = "vmax"))]
impl<D> VMaxDependencies for D {}

/// Stands in for the Voxel Json dependencies without the `voxj` feature.
/// Every type implements it.
#[cfg(not(feature = "voxj"))]
pub trait VoxjDependencies {}

#[cfg(not(feature = "voxj"))]
impl<D> VoxjDependencies for D {}
