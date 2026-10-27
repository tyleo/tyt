// Public API

#[allow(clippy::module_inception)]
mod palette;
mod palette_command;
mod palette_edit;
mod palette_list;
mod palette_quantize;
mod palette_show;

pub use palette::*;
pub use palette_command::*;
pub use palette_edit::*;
pub use palette_list::*;
pub use palette_quantize::*;
pub use palette_show::*;

// Internal API

mod internal;
pub(crate) use internal::*;
