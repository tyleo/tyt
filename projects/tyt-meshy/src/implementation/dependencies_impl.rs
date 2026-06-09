use crate::{
    Dependencies, Error, MeshRequest, MeshTask, MeshTaskFile, Result, TextureRequest,
    TextureTaskFile, UsrPrefs, implementation::meshy,
};
use std::{
    env, fs,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};
use tyt_preferences::load_user_git_prefs;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn meshy_api_key(&self) -> Result<Option<String>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let prefs: Option<UsrPrefs> =
            load_user_git_prefs(&prefs_deps, "meshy").map_err(Error::IO)?;
        Ok(prefs.and_then(|prefs| prefs.api_key))
    }

    fn current_dir(&self) -> Result<PathBuf> {
        Ok(env::current_dir()?)
    }

    fn read_text(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        String::from_utf8(bytes).map_err(|e| Error::IO(IOError::new(ErrorKind::InvalidData, e)))
    }

    fn read_input_task_id(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path).map_err(Error::IO)?;
        meshy::read_task_id(&bytes)
    }

    fn create_task(&self, api_key: &str, request: &MeshRequest) -> Result<String> {
        meshy::create_task(api_key, request)
    }

    fn create_texture_task(&self, api_key: &str, request: &TextureRequest) -> Result<String> {
        meshy::create_texture_task(api_key, request)
    }

    fn get_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask> {
        meshy::get_task(api_key, task_id)
    }

    fn get_texture_task(&self, api_key: &str, task_id: &str) -> Result<MeshTask> {
        meshy::get_texture_task(api_key, task_id)
    }

    fn download(&self, url: &str) -> Result<Vec<u8>> {
        meshy::download(url)
    }

    fn write_file(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path, bytes)?)
    }

    fn write_task_file(&self, path: &Path, file: &MeshTaskFile) -> Result<()> {
        let bytes = meshy::mesh_task_file_bytes(file)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_texture_task_file(&self, path: &Path, file: &TextureTaskFile) -> Result<()> {
        let bytes = meshy::texture_task_file_bytes(file)?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn sleep(&self, seconds: u64) -> Result<()> {
        thread::sleep(Duration::from_secs(seconds));
        Ok(())
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
