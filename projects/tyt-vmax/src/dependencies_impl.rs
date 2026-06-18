use crate::{Dependencies, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tyt_injection::serde_json::Value;
use vmax::{VMaxMaterial, VMaxScene, VMaxVoxel};
use vmax_serde::{VMaxSceneSerde, VXMaterialPaletteSerde, VXObjectDataSerde};

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

#[derive(Clone, Copy, Debug, Default)]
pub struct DependenciesImpl;

impl Dependencies for DependenciesImpl {
    fn copy_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        Ok(tyt_injection::copy_dir(src, dst)?)
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        Ok(tyt_injection::list_dir(path)?)
    }

    fn load_palette(&self, png_bytes: &[u8]) -> Result<Vec<[u8; 4]>> {
        let (rgba, _w, _h) = tyt_injection::decode_image_rgba(png_bytes)?;
        Ok(rgba
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect())
    }

    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
        Ok(tyt_injection::match_glob(pattern, candidates)?)
    }

    fn parse_materials(&self, vmaxpsb_bytes: &[u8]) -> Result<Vec<VMaxMaterial>> {
        let palette: VXMaterialPaletteSerde = tyt_injection::parse_bplist(vmaxpsb_bytes)?;
        Ok(palette.materials())
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

    fn parse_scene(&self, bytes: &[u8]) -> Result<VMaxScene> {
        let scene_serde: VMaxSceneSerde = tyt_injection::parse_json(bytes)?;
        Ok(scene_serde.into())
    }

    fn parse_voxels(&self, vmaxb_bytes: &[u8]) -> Result<Vec<VMaxVoxel>> {
        let decompressed = tyt_injection::lzfse_decompress(vmaxb_bytes);
        let data: VXObjectDataSerde = tyt_injection::parse_bplist(&decompressed)?;
        Ok(data.voxels())
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

    fn write_stdout(&self, contents: &[u8]) -> Result<()> {
        Ok(tyt_injection::write_stdout(contents)?)
    }
}
