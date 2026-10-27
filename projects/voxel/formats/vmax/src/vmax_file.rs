use crate::{
    VMaxContentsVmaxbFile, VMaxImage, VMaxOpaqueFile, VMaxPalettePngFile,
    VMaxPaletteSettingsVmaxpsbFile, VMaxSceneJsonFile, VMaxSelectionVmaxbFile,
};
use std::collections::BTreeMap;

/// Parsed contents of a `.vmax` package directory: `scene.json` plus every
/// other file the package holds but its undo history, which Voxel Max opens a
/// package without. The on-disk model a package round-trips through. Voxel
/// geometry and palette colors stay in their stored form here (snapshots,
/// packed bytes), decoded on demand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxFile {
    /// `scene.json`.
    pub scene_json_file: VMaxSceneJsonFile,

    /// `contents*.vmaxb` objects, keyed by filename.
    pub contents_files: BTreeMap<String, VMaxContentsVmaxbFile>,

    /// `palette*.settings.vmaxpsb` palettes, keyed by filename.
    pub palette_settings_files: BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,

    /// `palette*.png` color tables, keyed by filename.
    pub palette_png_files: BTreeMap<String, VMaxPalettePngFile>,

    /// `*.selection.vmaxb` saved voxel selections, keyed by filename.
    pub selection_vmaxb_files: BTreeMap<String, VMaxSelectionVmaxbFile>,

    /// Package-level `QuickLook/Thumbnail.png` preview, decoded. `None` when
    /// the package ships no thumbnail.
    pub thumbnail_png: Option<VMaxImage>,

    /// Per-object `QuickLook/contents*.vmaxb.png` previews, decoded, keyed by
    /// the object's `data` filename (e.g. `contents1.vmaxb`).
    pub contents_vmax_pngs: BTreeMap<String, VMaxImage>,

    /// Per-group `QuickLook/<group-id>.png` previews, decoded, keyed by the
    /// group's id (the `QuickLook/`-stripped, `.png`-stripped filename).
    pub group_pngs: BTreeMap<String, VMaxImage>,

    /// Every other file the package holds, keyed by its package path, as
    /// stored: animations, import records, external meshes and their
    /// textures, and any file a later Voxel Max adds.
    pub other_files: BTreeMap<String, VMaxOpaqueFile>,
}
