use crate::{
    Dependencies, Error, HierarchyBounds, HierarchyEntry, HierarchyTransform, MeshWithUvs, Result,
};
use std::{
    ffi::OsStr,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
    result::Result as StdResult,
};
use ty_math::{TySrgbaF32, TyVector3F64};
use tyt_injection::serde_json::Value;

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
        points: &[TyVector3F64],
        colors: &[Vec<TySrgbaF32>],
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
        let entries: Vec<Value> = tyt_injection::parse_json(json)?;
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

    fn parse_hierarchy_payloads_json(&self, json: &[u8]) -> Result<Vec<HierarchyEntry>> {
        let entries: Vec<Value> = tyt_injection::parse_json(json)?;
        entries
            .into_iter()
            .map(|entry| {
                Ok(HierarchyEntry {
                    name: payload_string(&entry, "name")?,
                    path: payload_string(&entry, "path")?,
                    object_type: payload_string(&entry, "type")?,
                    transform: entry.get("transform").map(payload_transform).transpose()?,
                    bounds: entry.get("bounds").map(payload_bounds).transpose()?,
                    extents: entry
                        .get("extents")
                        .map(|extents| component_strings(extents, "extents"))
                        .transpose()?,
                })
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

fn payload_string(entry: &Value, key: &str) -> StdResult<String, IOError> {
    entry[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| IOError::new(ErrorKind::InvalidData, format!("missing '{key}'")))
}

fn payload_transform(transform: &Value) -> StdResult<HierarchyTransform, IOError> {
    Ok(HierarchyTransform {
        position: component_strings(&transform["position"], "position")?,
        rotation: component_strings(&transform["rotation"], "rotation")?,
        scale: component_strings(&transform["scale"], "scale")?,
    })
}

fn payload_bounds(bounds: &Value) -> StdResult<HierarchyBounds, IOError> {
    Ok(HierarchyBounds {
        min: component_strings(&bounds["min"], "min")?,
        max: component_strings(&bounds["max"], "max")?,
    })
}

fn component_strings(value: &Value, key: &str) -> StdResult<[String; 3], IOError> {
    value
        .as_array()
        .and_then(|components| match components.as_slice() {
            [Value::String(x), Value::String(y), Value::String(z)] => {
                Some([x.clone(), y.clone(), z.clone()])
            }
            _ => None,
        })
        .ok_or_else(|| {
            IOError::new(
                ErrorKind::InvalidData,
                format!("expected 3 string components in '{key}'"),
            )
        })
}

#[cfg(test)]
mod tests {
    use crate::{Dependencies, DependenciesImpl};

    #[test]
    fn hierarchy_payloads_parse_with_their_optional_sections() {
        let json = r#"[
            {"name": "Rig", "path": "Rig", "type": "EMPTY",
             "transform": {"position": ["1.00", "2.00", "3.00"],
                           "rotation": ["0.00", "0.00", "0.00"],
                           "scale": ["1.00", "1.00", "1.00"]},
             "bounds": {"min": ["-1.00", "-1.00", "-1.00"],
                        "max": ["1.00", "1.00", "1.00"]},
             "extents": ["2.00", "2.00", "2.00"]},
            {"name": "Probe", "path": "Rig/Probe", "type": "EMPTY"}
        ]"#;

        let entries = DependenciesImpl
            .parse_hierarchy_payloads_json(json.as_bytes())
            .unwrap();

        let rig = &entries[0];
        assert_eq!(rig.object_type, "EMPTY");
        let transform = rig.transform.as_ref().unwrap();
        assert_eq!(transform.position, ["1.00", "2.00", "3.00"]);
        assert_eq!(rig.bounds.as_ref().unwrap().max, ["1.00", "1.00", "1.00"]);
        assert_eq!(rig.extents.as_ref().unwrap(), &["2.00", "2.00", "2.00"]);

        let probe = &entries[1];
        assert!(probe.transform.is_none() && probe.bounds.is_none() && probe.extents.is_none());
    }

    #[test]
    fn a_short_component_array_is_a_parse_error() {
        let json = r#"[{"name": "Rig", "path": "Rig", "type": "EMPTY", "extents": ["1.00"]}]"#;
        assert!(
            DependenciesImpl
                .parse_hierarchy_payloads_json(json.as_bytes())
                .is_err()
        );
    }
}
