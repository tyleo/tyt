use crate::{
    ColorFormat, Dependencies, ResolvedNodeTransform, Result, VMaxSceneNode, VoxjEncoding,
    VoxjFormat,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use ty_math::{
    TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3F64, ZERO_LENGTH_TOLERANCE,
};
use tyt_injection::serde_json::Value;
use vmax_codec::{DependenciesImpl as VMaxDependenciesImpl, from_scene_json_file_bytes};

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

/// A node's raw local transform: its stored translation and scale with the
/// `[x, y, z, angle]` axis-angle rotation decoded to a quaternion.
fn local_transform(node: &VMaxSceneNode) -> TyTransformF64 {
    let [x, y, z, angle] = node.rotation;
    let axis = TyVector3F64::new(x, y, z);
    let rotation = if axis.length() < ZERO_LENGTH_TOLERANCE {
        // A degenerate (zero) axis decodes to identity.
        TyQuaternionF64::IDENTITY
    } else {
        TyQuaternionF64::from_axis_angle(axis.normalize(), angle)
    };
    TyTransformF64::new(
        TyVector3F64::from_array(node.position),
        rotation,
        TyVector3F64::from_array(node.scale),
    )
}

/// The world transform of node `index`, composing local transforms down the
/// parent chain in `parent_of` and memoizing each result in `cache`.
fn world_transform(
    index: usize,
    nodes: &[VMaxSceneNode],
    parent_of: &[Option<usize>],
    cache: &mut [Option<TyTransformF64>],
) -> TyTransformF64 {
    if let Some(cached) = cache[index] {
        return cached;
    }
    let local = local_transform(&nodes[index]);
    let world = match parent_of[index] {
        Some(parent) => world_transform(parent, nodes, parent_of, cache).compose(&local),
        None => local,
    };
    cache[index] = Some(world);
    world
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

    fn scene_nodes(&self, scene_bytes: &[u8]) -> Result<Vec<VMaxSceneNode>> {
        let scene = from_scene_json_file_bytes(&VMaxDependenciesImpl, scene_bytes)?;
        let mut nodes = Vec::with_capacity(scene.groups.len() + scene.objects.len());
        for group in &scene.groups {
            nodes.push(VMaxSceneNode {
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
            nodes.push(VMaxSceneNode {
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

    fn resolve_node_transforms(
        &self,
        nodes: &[VMaxSceneNode],
        parent_of: &[Option<usize>],
        world: bool,
    ) -> Vec<ResolvedNodeTransform> {
        let mut cache = vec![None; nodes.len()];
        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                let transform = if world {
                    world_transform(index, nodes, parent_of, &mut cache)
                } else {
                    local_transform(node)
                };
                ResolvedNodeTransform {
                    position: transform.position.to_array(),
                    rotation: transform.rotation.to_euler_radians().to_array(),
                    scale: transform.scale.to_array(),
                }
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use crate::{Dependencies, DependenciesImpl, VMaxSceneNode};
    use std::f64::consts::PI;

    /// A group node with the given parent, position, and axis-angle
    /// rotation and unit scale.
    fn node(
        id: &str,
        parent: Option<&str>,
        position: [f64; 3],
        rotation: [f64; 4],
    ) -> VMaxSceneNode {
        VMaxSceneNode {
            id: id.to_string(),
            name: id.to_string(),
            parent_id: parent.map(str::to_string),
            is_group: true,
            position,
            rotation,
            scale: [1.0, 1.0, 1.0],
            bounds: None,
        }
    }

    #[test]
    fn local_transforms_decode_axis_angle_to_euler() {
        // A quarter turn about z reads on the z euler component alone.
        let nodes = vec![node("a", None, [1.0, 2.0, 3.0], [0.0, 0.0, 1.0, PI / 2.0])];
        let resolved = DependenciesImpl.resolve_node_transforms(&nodes, &[None], false);
        assert_eq!(resolved[0].position, [1.0, 2.0, 3.0]);
        assert!(resolved[0].rotation[0].abs() < 1e-9);
        assert!(resolved[0].rotation[1].abs() < 1e-9);
        assert!((resolved[0].rotation[2] - PI / 2.0).abs() < 1e-9);
    }

    #[test]
    fn world_transforms_compose_down_the_parent_chain() {
        // Parent: a quarter turn about z at +x. Its own +x child sits one unit
        // along the parent's rotated x, world +y, plus the parent translation,
        // and inherits the parent's turn.
        let nodes = vec![
            node("parent", None, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0, PI / 2.0]),
            node(
                "child",
                Some("parent"),
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ),
        ];
        let parent_of = [None, Some(0)];

        let local = DependenciesImpl.resolve_node_transforms(&nodes, &parent_of, false);
        assert!((local[1].position[0] - 1.0).abs() < 1e-9 && local[1].position[1].abs() < 1e-9);

        let world = DependenciesImpl.resolve_node_transforms(&nodes, &parent_of, true);
        assert!((world[1].position[0] - 1.0).abs() < 1e-9);
        assert!((world[1].position[1] - 1.0).abs() < 1e-9);
        assert!((world[1].rotation[2] - PI / 2.0).abs() < 1e-9);
    }
}
