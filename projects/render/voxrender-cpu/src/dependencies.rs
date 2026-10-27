use crate::MapRows;

/// The dependencies [`render`](crate::render) takes. A type implementing
/// [`MapRows`] implements this one.
pub trait Dependencies: MapRows {}

impl<D: MapRows> Dependencies for D {}
