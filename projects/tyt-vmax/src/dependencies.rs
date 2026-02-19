use crate::Result;
use std::path::{Path, PathBuf};
use vmax::VMaxScene;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;
    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;
    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>>;
    fn pack_scene_json(&self, scene_bytes: &[u8]) -> Result<Vec<u8>>;
    fn parse_scene(&self, bytes: &[u8]) -> Result<VMaxScene>;
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    fn remove_file(&self, path: &Path) -> Result<()>;
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
