//! Palette operations.

// Public API

mod edit_palettes;
mod palette_list;
mod palette_show;
mod quantize_palette;

pub use edit_palettes::*;
pub use palette_list::*;
pub use palette_show::*;
pub use quantize_palette::*;

// Internal API

mod error_palette_ext;
