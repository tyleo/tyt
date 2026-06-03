use crate::{
    Conv, Dependencies, Error, OaiRequest, OaiResponse, Result, implementation::openai,
    utilities::UsrPrefs,
};
use std::{
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
};
use tyt_injection::serde_json;
use tyt_preferences::{Dependencies as _, DeserializePrefs as _};

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn oai_api_key(&self) -> Result<Option<String>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let prefs: Option<UsrPrefs> =
            tyt_preferences::load_user_git_prefs(&prefs_deps, "oai").map_err(Error::IO)?;
        Ok(prefs.and_then(|p| p.api_key))
    }

    fn system_prompts_dir(&self) -> Result<Option<PathBuf>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let Some(git_root) = prefs_deps.git_root_dir().map_err(Error::IO)? else {
            return Ok(None);
        };

        let config = prefs_deps
            .read_file(&git_root.join(".tytconfig"))
            .map_err(Error::IO)?;
        let usr_config = prefs_deps
            .read_file(&git_root.join(".tytusrconfig"))
            .map_err(Error::IO)?;

        // `.tytconfig` is read first so a `systemPromptsDir` in `.tytusrconfig`
        // overrides it.
        let mut dir = None;
        for bytes in [config, usr_config].into_iter().flatten() {
            let prefs: Option<UsrPrefs> =
                UsrPrefs::deserialize_prefs(&bytes, "oai").map_err(Error::IO)?;
            if let Some(configured) = prefs
                .and_then(|prefs| prefs.img)
                .and_then(|img| img.system_prompts_dir)
            {
                dir = Some(configured);
            }
        }

        Ok(dir.map(|dir| {
            let path = PathBuf::from(&dir);
            if path.is_absolute() {
                path
            } else {
                git_root.join(path)
            }
        }))
    }

    fn read_system_prompt(&self, path: &Path) -> Result<Option<String>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let Some(bytes) = prefs_deps.read_file(path).map_err(Error::IO)? else {
            return Ok(None);
        };
        let text = String::from_utf8(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(Some(text))
    }

    fn read_stdin(&self) -> Result<String> {
        Ok(tyt_injection::read_stdin()?)
    }

    fn read_conv(&self, path: &Path) -> Result<Option<Conv>> {
        let prefs_deps = tyt_preferences::DependenciesImpl;
        let Some(bytes) = prefs_deps.read_file(path).map_err(Error::IO)? else {
            return Ok(None);
        };
        let conv =
            serde_json::from_slice(&bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(Some(conv))
    }

    fn write_conv(&self, path: &Path, conv: &Conv) -> Result<()> {
        let bytes =
            serde_json::to_vec_pretty(conv).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
        Ok(tyt_injection::write_file_atomic(path, &bytes)?)
    }

    fn write_image(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file_atomic(path, bytes)?)
    }

    fn generate_image(&self, api_key: &str, request: &OaiRequest) -> Result<OaiResponse> {
        openai::send_image_request(api_key, request)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }

    fn display_image_in_terminal(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::display_image_in_terminal(path)?)
    }
}
