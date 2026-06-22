use crate::{
    ColorFormat, Dependencies, Error, Result, VoxelMaxExt, VoxelMaxNode, VoxelMaxPalette,
    object_state_from_data,
};
use std::{
    collections::HashMap,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
};
use tyt_injection::serde_json::{self, Map, Value};
use vmax::VMaxSceneJsonFile;
use vmax_codec::{
    MaterialPalette, Voxel, decode_material_palette, decode_snapshots, from_palette_png_bytes,
    from_scene_json_bytes, from_vmaxb_bytes, from_vmaxpsb_bytes,
};
use voxj::VoxjValue;

/// Fallback `hist` reference for objects without a recognizable `contents`
/// reference. Voxel Max refuses to open a scene whose objects have an empty
/// `hist`, so every object must point at a history file name even though
/// packing leaves none on disk.
const PACKED_HIST: &str = "history.vmaxhb";

/// `scene.json` node keys the voxj document already represents natively, dropped
/// from the `voxel-max` provenance so they aren't duplicated in each node's
/// `extra`: name (`n` / group `name`), position (`t_p`), scale (`t_s`), and the
/// `data` / `pal` / `hist` filenames (regenerated on reconstruction).
const NATIVE_NODE_KEYS: [&str; 7] = ["n", "name", "t_p", "t_s", "data", "pal", "hist"];

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
        Ok(from_palette_png_bytes(png_bytes)?.0)
    }

    fn match_glob(&self, pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
        Ok(tyt_injection::match_glob(pattern, candidates)?)
    }

    fn parse_material_palette(&self, vmaxpsb_bytes: &[u8]) -> Result<MaterialPalette> {
        Ok(decode_material_palette(&from_vmaxpsb_bytes(vmaxpsb_bytes)?))
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

    fn parse_scene(&self, bytes: &[u8]) -> Result<VMaxSceneJsonFile> {
        Ok(from_scene_json_bytes(bytes)?)
    }

    fn parse_voxels(&self, vmaxb_bytes: &[u8]) -> Result<Vec<Voxel>> {
        Ok(decode_snapshots(&from_vmaxb_bytes(vmaxb_bytes)?.snapshots))
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

    fn voxel_max_ext(
        &self,
        scene_bytes: &[u8],
        palette_names: &[Option<String>],
        object_vmaxb: &[Option<Vec<u8>>],
    ) -> Result<VoxjValue> {
        let invalid = |e| -> Error { IOError::new(ErrorKind::InvalidData, e).into() };
        let mut value: Value = tyt_injection::parse_json(scene_bytes)?;
        let nodes = |value: &mut Value, key: &str| -> Result<Vec<VoxelMaxNode>> {
            let Some(Value::Array(array)) = value.as_object_mut().and_then(|map| map.remove(key))
            else {
                return Ok(Vec::new());
            };
            array
                .into_iter()
                .map(|mut node| {
                    // Drop fields the voxj document already carries natively so they
                    // don't fall through into `VoxelMaxNode::extra`.
                    if let Some(map) = node.as_object_mut() {
                        for key in NATIVE_NODE_KEYS {
                            map.remove(key);
                        }
                    }
                    serde_json::from_value(node).map_err(invalid)
                })
                .collect()
        };
        // Aligned with `main.hierarchyNodes`: groups first, then objects.
        let mut hierarchy_nodes = nodes(&mut value, "groups")?;
        hierarchy_nodes.extend(nodes(&mut value, "objects")?);
        let scene = serde_json::from_value(value).map_err(invalid)?;

        let palettes = palette_names
            .iter()
            .map(|name| name.clone().map(|name| VoxelMaxPalette { name }))
            .collect();
        // Capture each object's `.vmaxb` editor state (everything but snapshots).
        let object_states = object_vmaxb
            .iter()
            .map(|vmaxb| match vmaxb {
                Some(bytes) => Ok(Some(object_state_from_data(&from_vmaxb_bytes(bytes)?))),
                None => Ok(None),
            })
            .collect::<Result<Vec<_>>>()?;
        let voxel_max = serde_json::to_value(VoxelMaxExt {
            scene,
            hierarchy_nodes,
            palettes,
            object_states,
        })
        .map_err(invalid)?;
        let mut ext = Map::new();
        ext.insert("voxel-max".to_owned(), voxel_max);
        serde_json::from_value(Value::Object(ext)).map_err(invalid)
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
