use crate::{from_palette_png_bytes, from_scene_json_bytes, from_vmaxb_bytes, from_vmaxpsb_bytes};
use std::{
    collections::BTreeMap,
    io::{Error as IOError, ErrorKind, Result},
};
use vmax::VMaxSerdeFile;

/// Reads a whole `.vmax` package into a [`VMaxSerdeFile`]. `resolve` returns the bytes
/// of a file in the package by name (`Ok(None)` when the file is absent), so
/// `from_vmax` needs no filesystem of its own — the caller supplies the directory
/// access. `scene.json` is required; each object's
/// `contents*.vmaxb` / `palette*.png` / `palette*.settings.vmaxpsb` is loaded on
/// demand and de-duplicated by filename, so instances that share a file load it
/// once.
pub fn from_vmax<R>(mut resolve: R) -> Result<VMaxSerdeFile>
where
    R: FnMut(&str) -> Result<Option<Vec<u8>>>,
{
    let scene_bytes = resolve("scene.json")?
        .ok_or_else(|| IOError::new(ErrorKind::NotFound, "scene.json is missing"))?;
    let scene_json_file = from_scene_json_bytes(&scene_bytes)?;

    let mut contents_files = BTreeMap::new();
    let mut palette_settings_files = BTreeMap::new();
    let mut palette_png_files = BTreeMap::new();

    for object in &scene_json_file.objects {
        if !object.data.is_empty() && !contents_files.contains_key(&object.data) {
            let bytes = resolve(&object.data)?.ok_or_else(|| {
                IOError::new(
                    ErrorKind::NotFound,
                    format!("referenced contents file {} is missing", object.data),
                )
            })?;
            contents_files.insert(object.data.clone(), from_vmaxb_bytes(&bytes)?);
        }

        if object.palette.is_empty() {
            continue;
        }

        // The `pal` reference names the `palette*.png`; the material sidecar is
        // the matching `palette*.settings.vmaxpsb`. Either may be absent.
        if !palette_png_files.contains_key(&object.palette)
            && let Some(bytes) = resolve(&object.palette)?
        {
            palette_png_files.insert(object.palette.clone(), from_palette_png_bytes(&bytes)?);
        }
        if let Some(sidecar) = settings_sidecar(&object.palette)
            && !palette_settings_files.contains_key(&sidecar)
            && let Some(bytes) = resolve(&sidecar)?
        {
            palette_settings_files.insert(sidecar, from_vmaxpsb_bytes(&bytes)?);
        }
    }

    Ok(VMaxSerdeFile {
        scene_json_file,
        contents_files,
        palette_settings_files,
        palette_png_files,
    })
}

/// The `palette*.settings.vmaxpsb` sidecar name for a `palette*.png` reference.
fn settings_sidecar(png: &str) -> Option<String> {
    png.strip_suffix(".png")
        .map(|stem| format!("{stem}.settings.vmaxpsb"))
}

#[cfg(test)]
mod tests {
    use super::from_vmax;
    use crate::{encode_object_data, to_vmax};
    use std::collections::{BTreeMap, HashMap};
    use vmax::{
        VMaxCodecVoxel, VMaxObject, VMaxPalettePngFile, VMaxSceneJsonFile, VMaxSerdeFile,
        VMaxSerdePaletteSettingsVmaxpsbFile,
    };

    fn object(name: &str, data: &str, palette: &str) -> VMaxObject {
        VMaxObject {
            name: name.to_owned(),
            data: data.to_owned(),
            palette: palette.to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: format!("id-{name}"),
            parent_id: None,
            position: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
            ind: [0; 3],
            s: None,
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            center: [0.0; 3],
            bounds_min: None,
            bounds_max: None,
        }
    }

    fn sample() -> VMaxSerdeFile {
        let scene_json_file = VMaxSceneJsonFile {
            objects: vec![object("a", "contents.vmaxb", "palette.png")],
            v: 4,
            ..Default::default()
        };

        let contents = encode_object_data(
            &[
                VMaxCodecVoxel {
                    position: [1, 2, 3],
                    material_idx: 1,
                    color_idx: 5,
                },
                VMaxCodecVoxel {
                    position: [40, 5, 9],
                    material_idx: 2,
                    color_idx: 7,
                },
            ],
            "uuid-a",
        );
        let settings = VMaxSerdePaletteSettingsVmaxpsbFile {
            name: "pal".to_owned(),
            lc: vec![0u8; 256],
            ..Default::default()
        };
        let png = VMaxPalettePngFile(vec![[1, 2, 3, 255], [4, 5, 6, 255], [7, 8, 9, 0]]);

        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents.vmaxb".to_owned(), contents);
        let mut palette_settings_files = BTreeMap::new();
        palette_settings_files.insert("palette.settings.vmaxpsb".to_owned(), settings);
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert("palette.png".to_owned(), png);

        VMaxSerdeFile {
            scene_json_file,
            contents_files,
            palette_settings_files,
            palette_png_files,
        }
    }

    #[test]
    fn round_trips_through_a_directory_map() {
        let file = sample();

        // `to_vmax` writes into an in-memory directory.
        let mut dir: HashMap<String, Vec<u8>> = HashMap::new();
        to_vmax(&file, |name, bytes| {
            dir.insert(name.to_owned(), bytes.to_vec());
            Ok(())
        })
        .unwrap();

        // `from_vmax` reads it back through a resolver over the same map.
        let read = from_vmax(|name| Ok(dir.get(name).cloned())).unwrap();
        assert_eq!(read, file);
    }
}
