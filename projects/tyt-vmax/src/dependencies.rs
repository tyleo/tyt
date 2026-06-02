use crate::Result;
use std::path::{Path, PathBuf};
use vmax::VMaxScene;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>>;
    /// Rewrites `data`/`pal` references according to the supplied `(old, new)`
    /// rename pairs and repoints each object's `hist` at `history{n}.vmaxhb`
    /// matching its renumbered `contents{n}` reference.
    fn pack_scene_json(
        &self,
        scene_bytes: &[u8],
        data_renames: &[(&str, &str)],
        pal_renames: &[(&str, &str)],
    ) -> Result<Vec<u8>>;
    fn parse_scene(&self, bytes: &[u8]) -> Result<VMaxScene>;
    /// Returns each object's `(data, pal)` reference strings, read leniently
    /// from raw JSON so objects missing optional fields still parse.
    fn scene_object_refs(&self, scene_bytes: &[u8]) -> Result<Vec<(String, String)>>;
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
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
