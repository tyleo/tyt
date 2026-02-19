use crate::{Dependencies, Result};
use std::path::Path;
use vmax::VMaxScene;
use vmax_serde::VMaxSceneSerde;

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
        let glob = globset::Glob::new(pattern)?.compile_matcher();
        Ok(candidates.iter().map(|c| glob.is_match(c)).collect())
    }

    fn parse_scene(&self, bytes: &[u8]) -> Result<VMaxScene> {
        let scene_serde: VMaxSceneSerde = serde_json::from_slice(bytes)?;
        Ok(scene_serde.into())
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(tyt_injection::read_file(path)?)
    }

    fn rename_scene_nodes_json(
        &self,
        scene_bytes: &[u8],
        group_ids: &[&str],
        object_ids: &[&str],
        new_name: &str,
    ) -> Result<Vec<u8>> {
        let mut value: serde_json::Value = serde_json::from_slice(scene_bytes)?;

        if let Some(groups) = value.get_mut("groups").and_then(|v| v.as_array_mut()) {
            for group_val in groups {
                if let Some(id) = group_val.get("id").and_then(|v| v.as_str()) {
                    if group_ids.contains(&id) {
                        group_val["name"] = serde_json::Value::String(new_name.to_owned());
                    }
                }
            }
        }

        if let Some(objects) = value.get_mut("objects").and_then(|v| v.as_array_mut()) {
            for object_val in objects {
                if let Some(id) = object_val.get("id").and_then(|v| v.as_str()) {
                    if object_ids.contains(&id) {
                        object_val["n"] = serde_json::Value::String(new_name.to_owned());
                    }
                }
            }
        }

        Ok(serde_json::to_vec_pretty(&value)?)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file(path, contents)?)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
