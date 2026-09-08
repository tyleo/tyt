mod read_goxl;
mod write_goxl;

pub(crate) use read_goxl::*;
pub(crate) use write_goxl::*;

#[cfg(feature = "ext")]
mod read_goxl_with_ext;

#[cfg(feature = "ext")]
mod write_goxl_with_ext;

#[cfg(feature = "ext")]
pub(crate) use read_goxl_with_ext::*;

#[cfg(feature = "ext")]
pub(crate) use write_goxl_with_ext::*;
