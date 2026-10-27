#[cfg(feature = "sdfj")]
use crate::sdfj::SdfjDependencies;

/// The codec dependencies the read and write functions take: each enabled
/// format's, through that format's dependencies trait. A type implementing
/// every enabled format's trait implements this one. A type holding the
/// dependencies in another value implements
/// [`ForwardDependencies`](crate::ForwardDependencies) instead.
pub trait Dependencies: SdfjDependencies {}

impl<D: SdfjDependencies> Dependencies for D {}

/// Stands in for the SDF Json dependencies without the `sdfj` feature. Every
/// type implements it.
#[cfg(not(feature = "sdfj"))]
pub trait SdfjDependencies {}

#[cfg(not(feature = "sdfj"))]
impl<D> SdfjDependencies for D {}
