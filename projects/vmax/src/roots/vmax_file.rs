use crate::{
    VMaxBackend, VMaxHistoryVmaxhbFile, VMaxHistoryVmaxhvsbFile, VMaxHistoryVmaxhvscFile,
    VMaxPalettePngFile, VMaxQuickLookPngFile, VMaxSceneJsonFile, VMaxSelectionVmaxbFile,
};
use std::collections::BTreeMap;

/// The parsed contents of a `.vmax` package directory, generic over the payload
/// representation (see [`VMaxBackend`]): the single `scene.json` plus every other
/// file the package holds, each keyed by its package-relative path. The codec
/// models every file kind Voxel Max writes, so a package round-trips through it
/// without dropping anything — the per-object `contents*.vmaxb`, the per-palette
/// `palette*.settings.vmaxpsb` / `palette*.png`, the undo history
/// (`*.vmaxhb` / `*.vmaxhvsb` / `*.vmaxhvsc`), the saved `*.selection.vmaxb`, and
/// the `QuickLook/*` thumbnails.
///
/// `scene_json_file`, the `palette*.png` color tables, and the history /
/// selection / QuickLook files are the same in both representations; only the
/// contents and palette-settings payloads switch between their raw and decoded
/// forms. The history / selection / QuickLook payloads are opaque streams the
/// codec preserves verbatim (byte for byte), not parsed structures.
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

    /// `*.vmaxhb` undo-history streams (including `scene.vmaxhb`), keyed by
    /// filename and preserved verbatim.
    pub history_vmaxhb_files: BTreeMap<String, VMaxHistoryVmaxhbFile>,

    /// `*.vmaxhvsb` history voxel-snapshot buffers, keyed by filename and
    /// preserved verbatim.
    pub history_vmaxhvsb_files: BTreeMap<String, VMaxHistoryVmaxhvsbFile>,

    /// `*.vmaxhvsc` history voxel-snapshot sidecars, keyed by filename and
    /// preserved verbatim.
    pub history_vmaxhvsc_files: BTreeMap<String, VMaxHistoryVmaxhvscFile>,

    /// `*.selection.vmaxb` saved voxel selections, keyed by filename and
    /// preserved verbatim.
    pub selection_vmaxb_files: BTreeMap<String, VMaxSelectionVmaxbFile>,

    /// `QuickLook/*.png` thumbnails, keyed by their `QuickLook/`-prefixed
    /// package-relative path and preserved verbatim.
    pub quick_look_png_files: BTreeMap<String, VMaxQuickLookPngFile>,
}
