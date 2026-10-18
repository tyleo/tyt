#![deny(rustdoc::broken_intra_doc_links)]

//! Data types for a Voxel Max `.vmax` package, rooted at [`VMaxFile`].

// Public API

pub mod dependencies;
pub mod palette;
pub mod snapshots;

mod common;
mod contents_vmaxb;
mod history_vmaxhb;
mod history_vmaxhvsb;
mod history_vmaxhvsc;
mod palette_png;
mod palette_settings_vmaxpsb;
mod scene_json;
mod selection_vmaxb;
mod vmax_file;

pub use common::*;
pub use contents_vmaxb::*;
pub use dependencies::*;
pub use history_vmaxhb::*;
pub use history_vmaxhvsb::*;
pub use history_vmaxhvsc::*;
pub use palette_png::*;
pub use palette_settings_vmaxpsb::*;
pub use scene_json::*;
pub use selection_vmaxb::*;
pub use vmax_file::*;

// Internal API

#[cfg(feature = "serde")]
mod find_non_finite;
#[cfg(feature = "serde")]
mod finite;
#[cfg(feature = "serde")]
mod json_value;

#[cfg(feature = "serde")]
pub(crate) use find_non_finite::*;
#[cfg(feature = "serde")]
pub(crate) use finite::*;
#[cfg(feature = "serde")]
pub(crate) use json_value::*;
