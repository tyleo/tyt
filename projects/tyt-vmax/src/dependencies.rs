use crate::{ColorFormat, Result};
use std::path::{Path, PathBuf};
use vmax::{VMaxCodecPaletteSettingsVmaxpsbFile, VMaxCodecVoxel, VMaxSceneJsonFile};
use voxj::VoxjValue;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    /// Decodes the RGBA cells of a `palette*.png` (one `[r, g, b, a]` per pixel).
    fn load_palette(&self, png_bytes: &[u8]) -> Result<Vec<[u8; 4]>>;
    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>>;
    /// Decodes the material palette (name, slots, RGBA color table, and the
    /// palette-level editor state) of a `palette*.settings.vmaxpsb` plist.
    fn parse_material_palette(
        &self,
        vmaxpsb_bytes: &[u8],
    ) -> Result<VMaxCodecPaletteSettingsVmaxpsbFile>;
    /// Rewrites `data`/`pal` references according to the supplied `(old, new)`
    /// rename pairs and repoints each object's `hist` at `history{n}.vmaxhb`
    /// matching its renumbered `contents{n}` reference.
    fn pack_scene_json(
        &self,
        scene_bytes: &[u8],
        data_renames: &[(&str, &str)],
        pal_renames: &[(&str, &str)],
    ) -> Result<Vec<u8>>;
    fn parse_scene(&self, bytes: &[u8]) -> Result<VMaxSceneJsonFile>;
    /// Decodes the voxels of a `contents*.vmaxb` payload (LZFSE-framed binary plist).
    fn parse_voxels(&self, vmaxb_bytes: &[u8]) -> Result<Vec<VMaxCodecVoxel>>;
    /// Returns each object's `(data, pal)` reference strings, read leniently
    /// from raw JSON so objects missing optional fields still parse.
    fn scene_object_refs(&self, scene_bytes: &[u8]) -> Result<Vec<(String, String)>>;
    /// Builds the `voxel-max` extension payload for `from-voxj` reconstruction
    /// and returns it as UTF-8 JSON bytes ready to embed at the voxj document's
    /// `main.ext`. Captures the Voxel Max scene/node state that has no native
    /// voxj home; unmodeled keys are preserved verbatim. `palette_names[i]` is
    /// the display name of `main.palettes[i]` when that palette is a material
    /// palette, aligned by index (`None` for color palettes). `object_vmaxb[i]`
    /// is the raw `contents*.vmaxb` bytes for `main.objects[i]` (`None` when the
    /// object had none); their editor state (`tools`/`brush`/`cam`,
    /// `uuid`/version) is captured so `from-voxj` can rebuild an importable
    /// package.
    fn voxel_max_ext(
        &self,
        scene_bytes: &[u8],
        palette_names: &[Option<String>],
        object_vmaxb: &[Option<Vec<u8>>],
    ) -> Result<VoxjValue>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    fn remove_file(&self, path: &Path) -> Result<()>;
    fn rename_file(&self, from: &Path, to: &Path) -> Result<()>;
    fn rename_scene_nodes_json(
        &self,
        scene_bytes: &[u8],
        group_ids: &[&str],
        object_ids: &[&str],
        new_name: &str,
    ) -> Result<Vec<u8>>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    /// Reconstructs a `.vmax` package directory at `output` from `.voxj` /
    /// `.voxjz` document bytes (the inverse of the `to-voxj` pipeline).
    /// `color_format` selects where each object's colors are stored: a
    /// `palette*.png` image ([`ColorFormat::Png`]), the material
    /// `palette*.settings.vmaxpsb` `colors` table ([`ColorFormat::Plist`]), or
    /// both ([`ColorFormat::All`]). The `pal` references and material sidecars
    /// are written in every case.
    fn write_vmax_package(
        &self,
        voxj_bytes: &[u8],
        output: &Path,
        color_format: ColorFormat,
    ) -> Result<()>;
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
