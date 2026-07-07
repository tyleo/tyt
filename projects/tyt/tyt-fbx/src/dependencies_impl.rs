use crate::{Dependencies, Error, MeshWithUvs, Result};
use std::{
    ffi::OsStr,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
    result::Result as StdResult,
};
use ty_math::{TyRgbaColor, TyVector3};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn create_temp_dir(&self) -> Result<PathBuf> {
        Ok(tyt_injection::create_temp_dir()?)
    }

    fn exec_blender_script<
        P1: AsRef<Path>,
        P2: AsRef<Path>,
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_dir: P1,
        script_py_path: P2,
        args: I,
    ) -> Result<Vec<u8>> {
        let blender_args = tyt_injection::Args::new()
            .arg("--background")
            .arg("--python-expr")
            .arg(format!(
                "import sys; sys.path.insert(0, r'{}')",
                script_dir.as_ref().display(),
            ))
            .arg("--python")
            .arg(script_py_path.as_ref())
            .arg("--")
            .args(args);

        tyt_injection::exec_map("blender", blender_args, Error::IO, Error::Blender)
    }

    fn parse_mesh_with_uvs_json(&self, json: &[u8]) -> Result<MeshWithUvs> {
        Ok(tyt_injection::parse_mesh_with_uvs_json(json)?)
    }

    fn serialize_points_and_colors_json(
        &self,
        points: &[TyVector3],
        colors: &[Vec<TyRgbaColor>],
    ) -> Result<Vec<u8>> {
        Ok(tyt_injection::serialize_points_and_colors_json(
            points, colors,
        )?)
    }

    fn load_image_rgba(&self, path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        Ok(tyt_injection::load_image_rgba(path)?)
    }

    fn display_image_in_terminal(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::display_image_in_terminal(path)?)
    }

    fn match_paths(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>> {
        Ok(tyt_injection::match_paths(patterns, candidates)?)
    }

    fn parse_hierarchy_json(&self, json: &[u8]) -> Result<Vec<(String, String, String)>> {
        let entries: Vec<tyt_injection::serde_json::Value> = tyt_injection::parse_json(json)?;
        entries
            .into_iter()
            .map(|v| {
                let name = v["name"]
                    .as_str()
                    .ok_or_else(|| IOError::new(ErrorKind::InvalidData, "missing 'name'"))?
                    .to_owned();
                let path = v["path"]
                    .as_str()
                    .ok_or_else(|| IOError::new(ErrorKind::InvalidData, "missing 'path'"))?
                    .to_owned();
                let obj_type = v["type"]
                    .as_str()
                    .ok_or_else(|| IOError::new(ErrorKind::InvalidData, "missing 'type'"))?
                    .to_owned();
                Ok((name, path, obj_type))
            })
            .collect::<StdResult<Vec<_>, IOError>>()
            .map_err(Error::from)
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Ok(tyt_injection::remove_dir_all(path.as_ref())?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn write_file<P: AsRef<Path>>(&self, path: P, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path.as_ref(), contents)?)
    }
}
