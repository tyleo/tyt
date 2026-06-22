use crate::{
    VMaxContentsVmaxbFile, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneJsonFile,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The parsed contents of a `.vmax` package directory: the single `scene.json`
/// plus the per-object `contents*.vmaxb` and per-palette
/// `palette*.settings.vmaxpsb` / `palette*.png` files it references by name.
///
/// Each file kind is its own `*File` type, named after the file it parses; the
/// per-instance files are keyed by their on-disk filename.
///
/// This container is assembled field by field by the codec from its parts, not
/// usually decoded with serde.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxFile {
    /// `scene.json`.
    pub scene_json_file: VMaxSceneJsonFile,

    /// `contents*.vmaxb` files, keyed by filename.
    pub contents_vmaxb_files: BTreeMap<String, VMaxContentsVmaxbFile>,

    /// `palette*.settings.vmaxpsb` files, keyed by filename.
    pub palette_settings_vmaxpsb_files: BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,

    /// `palette*.png` files, keyed by filename.
    pub palette_png_files: BTreeMap<String, VMaxPalettePngFile>,
}
