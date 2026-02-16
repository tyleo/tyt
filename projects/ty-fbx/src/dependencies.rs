use crate::{Result, Script};
use std::{
    ffi::OsStr,
    iter,
    path::{Path, PathBuf},
};

pub trait Dependencies {
    fn create_temp_dir(&self) -> Result<PathBuf>;

    fn exec_blender_script<P: AsRef<Path>, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        script_py_path: P,
        args: I,
    ) -> Result<Vec<u8>>;

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    fn write_file<P: AsRef<Path>>(&self, path: P, contents: &[u8]) -> Result<()>;

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

            self.exec_blender_script(script_py_path, args)
        })();

        let output = result?;
        self.remove_dir_all(&temp_dir)?;

        Ok(output)
    }

    fn exec_temp_blender_script<'a, I: IntoIterator<Item = S>, S: AsRef<OsStr>>(
        &self,
        script_py: &'a Script<'a>,
        args: I,
    ) -> Result<Vec<u8>> {
        self.exec_temp_blender_scripts(script_py, iter::empty(), args)
    }
}
