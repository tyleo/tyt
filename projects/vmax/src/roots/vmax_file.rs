use crate::{VMaxBackend, VMaxPalettePngFile, VMaxSceneJsonFile};
use std::collections::BTreeMap;

/// The parsed contents of a `.vmax` package directory, generic over the payload
/// representation (see [`VMaxBackend`]): the single `scene.json` plus the
/// per-object `contents*.vmaxb` and per-palette `palette*.settings.vmaxpsb` /
/// `palette*.png` files it references by name, keyed by their on-disk filename.
///
/// `scene_json_file` and the `palette*.png` color tables are the same in both
/// representations; only the contents and palette-settings payloads switch
/// between their raw and decoded forms.
///
/// This container is assembled field by field by the codec from its parts and is
/// never decoded with serde, so it carries no serde derives; its component
/// `*File` fields keep theirs.
#[derive(Clone, Debug, PartialEq)]
pub struct VMaxFile<Backend: VMaxBackend> {
    /// `scene.json`.
    pub scene_json_file: VMaxSceneJsonFile,

    /// `contents*.vmaxb` objects, keyed by filename.
    pub contents_files: BTreeMap<String, Backend::Contents>,

    /// `palette*.settings.vmaxpsb` palettes, keyed by filename.
    pub palette_settings_files: BTreeMap<String, Backend::PaletteSettings>,

    /// `palette*.png` color tables, keyed by filename.
    pub palette_png_files: BTreeMap<String, VMaxPalettePngFile>,
}
