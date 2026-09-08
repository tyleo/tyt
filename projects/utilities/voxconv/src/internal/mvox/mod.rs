mod read_mvox;
mod write_mvox;

pub(crate) use read_mvox::*;
pub(crate) use write_mvox::*;

#[cfg(feature = "ext")]
mod read_mvox_with_ext;

#[cfg(feature = "ext")]
mod write_mvox_with_ext;

#[cfg(feature = "ext")]
pub(crate) use read_mvox_with_ext::*;

#[cfg(feature = "ext")]
pub(crate) use write_mvox_with_ext::*;
