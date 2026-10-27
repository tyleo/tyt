// Public API

mod geometry;

pub use geometry::*;

// Optional API

#[cfg(feature = "color")]
mod color;

#[cfg(feature = "color")]
pub use color::*;

// Test support

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::*;
