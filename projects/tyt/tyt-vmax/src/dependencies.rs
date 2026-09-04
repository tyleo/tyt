use crate::{ColorFormat, ResolvedNodeTransform, Result, VMaxSceneNode, VoxjEncoding, VoxjFormat};
use std::path::{Path, PathBuf};

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()>;

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    fn match_paths(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>>;

    /// Like [`match_paths`](Self::match_paths) but with real gitignore
    /// directory semantics: a matched directory pulls in its whole subtree.
    fn match_subtrees(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>>;

    /// Rewrites `data`/`pal` references according to the supplied `(old, new)`
    /// rename pairs and repoints each object's `hist` at `history{n}.vmaxhb`
    /// matching its renumbered `contents{n}` reference.
    fn pack_scene_json(
        &self,
        scene_bytes: &[u8],
        data_renames: &[(&str, &str)],
        pal_renames: &[(&str, &str)],
    ) -> Result<Vec<u8>>;

    /// Flattens a `scene.json`'s hierarchy into one [`VMaxSceneNode`] per
    /// group and object (groups first), exposing only the identity and parentage
    /// the `hierarchy` and `rename-node` commands need.
    fn scene_nodes(&self, scene_bytes: &[u8]) -> Result<Vec<VMaxSceneNode>>;

    /// Resolves each node's transform for the `hierarchy` command's
    /// `--show-transforms`, one entry per node in `nodes` order, with rotation
    /// returned as euler angles in radians. With `world` set, each transform is
    /// composed down its parent chain through `parent_of`, which holds the
    /// parent index of each node or `None` at a root; otherwise it is the
    /// node's raw local transform.
    fn resolve_node_transforms(
        &self,
        nodes: &[VMaxSceneNode],
        parent_of: &[Option<usize>],
        world: bool,
    ) -> Vec<ResolvedNodeTransform>;

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

    /// Converts the `.vmax` package directory at `input` into a Voxel Json
    /// document written to stdout: decodes the package, translates it to the
    /// voxj model, then block-encodes it with `encoding` and serializes it in
    /// the container `format` selects.
    fn write_voxj(&self, input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()>;

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
