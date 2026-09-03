//! Unpacks the color table a
//! [`VMaxPaletteSettingsVmaxpsbFile`](crate::VMaxPaletteSettingsVmaxpsbFile)
//! embeds when a palette ships no sibling `palette*.png`.

mod decode_palette_colors;
mod error;
mod result;

pub use decode_palette_colors::*;
pub use error::*;
pub use result::*;
