mod package_file;
mod read_vmax;
mod write_vmax;

pub(crate) use package_file::*;
pub(crate) use read_vmax::*;
pub(crate) use write_vmax::*;

#[cfg(feature = "ext")]
mod read_vmax_with_ext;

#[cfg(feature = "ext")]
mod write_vmax_with_ext;

#[cfg(feature = "ext")]
pub(crate) use read_vmax_with_ext::*;

#[cfg(feature = "ext")]
pub(crate) use write_vmax_with_ext::*;
