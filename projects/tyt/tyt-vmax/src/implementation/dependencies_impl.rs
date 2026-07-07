use crate::{ColorFormat, Dependencies, Result, VoxelMaxSceneNode, VoxjEncoding, VoxjFormat};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tyt_injection::serde_json::Value;
use vmax_codec::from_scene_json_file_bytes;

/// Fallback `hist` reference for objects without a recognizable `contents`
/// reference. Voxel Max refuses to open a scene whose objects have an empty
/// `hist`, so every object must point at a history file name even though
/// packing leaves none on disk.
const PACKED_HIST: &str = "history.vmaxhb";

/// Replaces a string `field` on `object_val` using `map`, if its current value is a key.
fn rename_field(object_val: &mut Value, field: &str, map: &HashMap<&str, &str>) {
    if let Some(current) = object_val.get(field).and_then(|v| v.as_str())
        && let Some(&new) = map.get(current)
    {
        object_val[field] = Value::String(new.to_owned());
    }
}

/// History file name for an object, mirroring the number of its (already
/// renumbered) `contents{n}.vmaxb` reference so each object points at
/// `history{n}.vmaxhb`. Objects without a recognizable `data` reference fall
/// back to the blank history name.
fn hist_for(object_val: &Value) -> String {
    object_val
        .get("data")
        .and_then(|v| v.as_str())
        .and_then(|data| data.strip_prefix("contents")?.strip_suffix(".vmaxb"))
        .map(|suffix| format!("history{suffix}.vmaxhb"))
        .unwrap_or_else(|| PACKED_HIST.to_owned())
}

/// Absolute authored box from a `center` and its center-relative `min`/`max`
/// corners, or `None` when either corner is absent.
fn authored_bounds(
    center: [f64; 3],
    min: Option<[f64; 3]>,
    max: Option<[f64; 3]>,
) -> Option<([f64; 3], [f64; 3])> {
    let (min, max) = (min?, max?);
    let corner = |offset: [f64; 3]| {
        [
            center[0] + offset[0],
            center[1] + offset[1],
            center[2] + offset[2],
        ]
    };
    Some((corner(min), corner(max)))
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        Ok(tyt_injection::copy_dir(src, dst)?)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        Ok(tyt_injection::list_dir(path)?)
    }

    fn match_paths(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>> {
        Ok(tyt_injection::match_paths(patterns, candidates)?)
    }

    fn match_subtrees(&self, patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>> {
        Ok(tyt_injection::match_subtrees(patterns, candidates)?)
    }

    fn pack_scene_json(
        &self,
        scene_bytes: &[u8],
        data_renames: &[(&str, &str)],
        pal_renames: &[(&str, &str)],
    ) -> Result<Vec<u8>> {
        let data_map: HashMap<&str, &str> = data_renames.iter().copied().collect();
        let pal_map: HashMap<&str, &str> = pal_renames.iter().copied().collect();
        let mut value: Value = tyt_injection::parse_json(scene_bytes)?;

        if let Some(objects) = value.get_mut("objects").and_then(|v| v.as_array_mut()) {
            for object_val in objects {
                rename_field(object_val, "data", &data_map);
                rename_field(object_val, "pal", &pal_map);
                object_val["hist"] = Value::String(hist_for(object_val));
            }
        }

        Ok(tyt_injection::serialize_json_pretty(&value)?)
    }

    fn scene_nodes(&self, scene_bytes: &[u8]) -> Result<Vec<VoxelMaxSceneNode>> {
        let scene = from_scene_json_file_bytes(scene_bytes)?;
        let mut nodes = Vec::with_capacity(scene.groups.len() + scene.objects.len());
        for group in &scene.groups {
            nodes.push(VoxelMaxSceneNode {
                id: group.id.clone(),
                name: group.name.clone(),
                parent_id: group.parent_id.clone(),
                is_group: true,
                position: group.position,
                rotation: group.rotation,
                scale: group.scale,
                bounds: authored_bounds(group.center, group.bounds_min, group.bounds_max),
            });
        }
        for object in &scene.objects {
            nodes.push(VoxelMaxSceneNode {
                id: object.id.clone(),
                name: object.name.clone(),
                parent_id: object.parent_id.clone(),
                is_group: false,
                position: object.position,
                rotation: object.rotation,
                scale: object.scale,
                bounds: authored_bounds(object.center, object.bounds_min, object.bounds_max),
            });
        }
        Ok(nodes)
    }

    fn scene_object_refs(&self, scene_bytes: &[u8]) -> Result<Vec<(String, String)>> {
        let value: Value = tyt_injection::parse_json(scene_bytes)?;
        let field = |object: &Value, key: &str| {
            object
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned()
        };
        Ok(value
            .get("objects")
            .and_then(|v| v.as_array())
            .map(|objects| {
                objects
                    .iter()
                    .map(|object| (field(object, "data"), field(object, "pal")))
                    .collect()
            })
            .unwrap_or_default())
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        Ok(tyt_injection::read_file(path)?)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        Ok(tyt_injection::remove_file(path)?)
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<()> {
        Ok(tyt_injection::rename_file(from, to)?)
    }

    fn rename_scene_nodes_json(
        &self,
        scene_bytes: &[u8],
        group_ids: &[&str],
        object_ids: &[&str],
        new_name: &str,
    ) -> Result<Vec<u8>> {
        let mut value: tyt_injection::serde_json::Value = tyt_injection::parse_json(scene_bytes)?;

        if let Some(groups) = value.get_mut("groups").and_then(|v| v.as_array_mut()) {
            for group_val in groups {
                if let Some(id) = group_val.get("id").and_then(|v| v.as_str())
                    && group_ids.contains(&id)
                {
                    group_val["name"] =
                        tyt_injection::serde_json::Value::String(new_name.to_owned());
                }
            }
        }

        if let Some(objects) = value.get_mut("objects").and_then(|v| v.as_array_mut()) {
            for object_val in objects {
                if let Some(id) = object_val.get("id").and_then(|v| v.as_str())
                    && object_ids.contains(&id)
                {
                    object_val["n"] = tyt_injection::serde_json::Value::String(new_name.to_owned());
                }
            }
        }

        Ok(tyt_injection::serialize_json_pretty(&value)?)
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_file(path, contents)?)
    }

    fn write_voxj(&self, input: &Path, encoding: VoxjEncoding, format: VoxjFormat) -> Result<()> {
        super::write_voxj::write_voxj(input, encoding, format)
    }

    fn write_vmax_package(
        &self,
        voxj_bytes: &[u8],
        output: &Path,
        color_format: ColorFormat,
    ) -> Result<()> {
        super::write_vmax_package::write_vmax_package(voxj_bytes, output, color_format)
    }

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
