// Public API

#[allow(clippy::module_inception)]
mod profile_palette;
mod profile_palette_command;
mod profile_palette_edit;
mod profile_palette_quantize;
mod profile_palette_show;

pub use profile_palette::*;
pub use profile_palette_command::*;
pub use profile_palette_edit::*;
pub use profile_palette_quantize::*;
pub use profile_palette_show::*;
