use crate::{MeshWithUvs, Result, Script, utilities::COMMON_PY};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use ty_math::{TySrgba, TyVector3};

pub trait Dependencies {
    fn create_temp_dir(&self) -> Result<PathBuf>;

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
    ) -> Result<Vec<u8>>;

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn write_file<P: AsRef<Path>>(&self, path: P, contents: &[u8]) -> Result<()>;

    fn write_stdout(&self, contents: &[u8]) -> Result<()>;

    fn parse_mesh_with_uvs_json(&self, json: &[u8]) -> Result<MeshWithUvs>;

    fn serialize_points_and_colors_json(
        &self,
        points: &[TyVector3],
        colors: &[Vec<TySrgba>],
    ) -> Result<Vec<u8>>;

    fn load_image_rgba(&self, path: &Path) -> Result<(Vec<u8>, u32, u32)>;

    fn display_image_in_terminal(&self, path: &Path) -> Result<()>;

    fn match_paths(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>>;

    fn parse_hierarchy_json(&self, json: &[u8]) -> Result<Vec<(String, String, String)>>;

    // --- Provided methods ---

    fn exec_temp_blender_scripts<
        'a,
        I1: IntoIterator<Item = &'a Script<'a>>,
        I2: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_py: &'a Script<'a>,
        additional_scripts: I1,
        args: I2,
    ) -> Result<Vec<u8>> {
        let temp_dir = self.create_temp_dir()?;

        let result = (|| {
            let script_py_path = temp_dir.join(script_py.relative_file_path);
            self.write_file(&script_py_path, script_py.content.as_bytes())?;

            for additional_script in additional_scripts.into_iter() {
                let additional_script_path = temp_dir.join(additional_script.relative_file_path);
                self.write_file(
                    &additional_script_path,
                    additional_script.content.as_bytes(),
                )?;
            }

            self.exec_blender_script(&temp_dir, script_py_path, args)
        })();

        let output = result?;
        self.remove_dir_all(&temp_dir)?;

        Ok(output)
    }

    fn exec_temp_blender_scripts_with_stdout<
        'a,
        I: IntoIterator<Item = &'a Script<'a>>,
        S: AsRef<OsStr>,
    >(
        &self,
        script_py: &'a Script<'a>,
        additional_scripts: I,
        args: impl IntoIterator<Item = S>,
    ) -> Result<()> {
        let stdout = self.exec_temp_blender_scripts(script_py, additional_scripts, args)?;
        self.write_stdout(&stdout)?;
        Ok(())
    }

    fn exec_temp_blender_script<'a, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        script_py: &'a Script<'a>,
        args: I,
    ) -> Result<Vec<u8>> {
        self.exec_temp_blender_scripts(script_py, [&COMMON_PY], args)
    }

    fn exec_temp_blender_script_with_stdout<'a, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        script_py: &'a Script<'a>,
        args: I,
    ) -> Result<()> {
        self.exec_temp_blender_scripts_with_stdout(script_py, [&COMMON_PY], args)
    }
}
