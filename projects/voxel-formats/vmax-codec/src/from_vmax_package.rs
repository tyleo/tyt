use crate::{
    DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse, Error, Result,
    from_contents_vmaxb_file_bytes, from_history_vmaxhb_file_bytes,
    from_history_vmaxhvsb_file_bytes, from_history_vmaxhvsc_file_bytes, from_image_png_file_bytes,
    from_palette_png_file_bytes, from_palette_settings_vmaxpsb_file_bytes,
    from_scene_json_file_bytes, from_selection_vmaxb_file_bytes,
};
use std::collections::BTreeMap;
use vmax::VMaxFile;

/// The package-level thumbnail's path within a `.vmax`.
const THUMBNAIL_PATH: &str = "QuickLook/Thumbnail.png";

/// Prefix every `QuickLook/` entry shares.
const QUICK_LOOK_PREFIX: &str = "QuickLook/";

/// Reads a whole `.vmax` package into a [`VMaxFile`], decoding each file
/// through `dependencies`. `scene.json` is required. A filename matching no
/// known kind is an error.
///
/// # Arguments
/// * `list` - returns the package-relative path of every file, so `QuickLook/`
///   entries keep their subdirectory prefix.
/// * `resolve` - returns a file's bytes by that path, or `Ok(None)` if it has
///   since vanished.
pub fn from_vmax_package<D, L, R>(dependencies: &D, list: L, mut resolve: R) -> Result<VMaxFile>
where
    D: DecompressLzfse + DecodeVMaxPlist + DecodePng + DecodeVMaxSceneJson,
    L: FnOnce() -> Result<Vec<String>>,
    R: FnMut(&str) -> Result<Option<Vec<u8>>>,
{
    let scene_bytes =
        resolve("scene.json")?.ok_or_else(|| Error::Invalid("scene.json is missing".to_owned()))?;
    let scene_json_file = from_scene_json_file_bytes(dependencies, &scene_bytes)?;

    let mut contents_files = BTreeMap::new();
    let mut palette_settings_files = BTreeMap::new();
    let mut palette_png_files = BTreeMap::new();
    let mut history_vmaxhb_files = BTreeMap::new();
    let mut history_vmaxhvsb_files = BTreeMap::new();
    let mut history_vmaxhvsc_files = BTreeMap::new();
    let mut selection_vmaxb_files = BTreeMap::new();
    let mut thumbnail_png = None;
    let mut contents_vmax_pngs = BTreeMap::new();
    let mut group_pngs = BTreeMap::new();

    // Classify every listed file by name and parse it into the matching map.
    // `scene.json` is already parsed above. `QuickLook/` thumbnails split by
    // role: the package `Thumbnail.png`, the per-object `contents*.vmaxb.png`,
    // and the per-group `<id>.png` (everything else under `QuickLook/`).
    // `.selection.vmaxb` is checked before `.vmaxb` so a selection sidecar is
    // not mistaken for an object. Every kind Voxel Max writes is modeled, so an
    // unrecognized name is reported rather than dropped.
    for path in list()? {
        if path == "scene.json" {
            continue;
        }
        let Some(bytes) = resolve(&path)? else {
            continue;
        };
        if let Some(name) = path.strip_prefix(QUICK_LOOK_PREFIX) {
            if path == THUMBNAIL_PATH {
                thumbnail_png = Some(from_image_png_file_bytes(dependencies, &bytes)?);
            } else if let Some(data) = name.strip_suffix(".png").and_then(strip_contents_suffix) {
                contents_vmax_pngs.insert(data, from_image_png_file_bytes(dependencies, &bytes)?);
            } else if let Some(id) = name.strip_suffix(".png") {
                group_pngs.insert(
                    id.to_owned(),
                    from_image_png_file_bytes(dependencies, &bytes)?,
                );
            } else {
                return Err(Error::Invalid(format!(
                    "unrecognized QuickLook file in .vmax package: {path}"
                )));
            }
        } else if path.ends_with(".selection.vmaxb") {
            selection_vmaxb_files.insert(path, from_selection_vmaxb_file_bytes(&bytes)?);
        } else if path.ends_with(".vmaxb") {
            contents_files.insert(path, from_contents_vmaxb_file_bytes(dependencies, &bytes)?);
        } else if path.ends_with(".settings.vmaxpsb") {
            palette_settings_files.insert(
                path,
                from_palette_settings_vmaxpsb_file_bytes(dependencies, &bytes)?,
            );
        } else if path.ends_with(".png") {
            palette_png_files.insert(path, from_palette_png_file_bytes(dependencies, &bytes)?);
        } else if path.ends_with(".vmaxhb") {
            history_vmaxhb_files
                .insert(path, from_history_vmaxhb_file_bytes(dependencies, &bytes)?);
        } else if path.ends_with(".vmaxhvsb") {
            history_vmaxhvsb_files.insert(
                path,
                from_history_vmaxhvsb_file_bytes(dependencies, &bytes)?,
            );
        } else if path.ends_with(".vmaxhvsc") {
            history_vmaxhvsc_files.insert(
                path,
                from_history_vmaxhvsc_file_bytes(dependencies, &bytes)?,
            );
        } else {
            return Err(Error::Invalid(format!(
                "unrecognized file in .vmax package: {path}"
            )));
        }
    }

    Ok(VMaxFile {
        scene_json_file,
        contents_files,
        palette_settings_files,
        palette_png_files,
        history_vmaxhb_files,
        history_vmaxhvsb_files,
        history_vmaxhvsc_files,
        selection_vmaxb_files,
        thumbnail_png,
        contents_vmax_pngs,
        group_pngs,
    })
}

/// Returns the object `data` filename a per-object thumbnail names (a
/// `contents*.vmaxb` stem), or `None` when the name is not a contents preview.
fn strip_contents_suffix(name: &str) -> Option<String> {
    name.ends_with(".vmaxb").then(|| name.to_owned())
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use super::from_vmax_package;
    use crate::{DependenciesImpl, to_vmax_package};
    use std::collections::{BTreeMap, HashMap};
    use vmax::{
        VMaxFile, VMaxHistorySession, VMaxHistoryVmaxhbFile, VMaxHistoryVmaxhvsbFile,
        VMaxHistoryVmaxhvscFile, VMaxImage, VMaxObject, VMaxPalettePngFile,
        VMaxPaletteSettingsVmaxpsbFile, VMaxSceneJsonFile, VMaxSelectionVmaxbFile, VMaxValue,
        snapshots::{VMaxVoxel, encode_contents_vmaxb_file_from_voxels},
    };

    fn object(name: &str, data: &str, palette: &str) -> VMaxObject {
        VMaxObject {
            name: name.to_owned(),
            data: data.to_owned(),
            palette: palette.to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: format!("id-{name}"),
            parent_id: None,
            hidden: None,
            position: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
            ind: [0; 3],
            s: None,
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [0.0; 3],
            bounds_min: None,
            bounds_max: None,
        }
    }

    fn image() -> VMaxImage {
        VMaxImage {
            width: 2,
            height: 1,
            pixels: vec![[1, 2, 3, 255], [4, 5, 6, 0]],
        }
    }

    fn sample() -> VMaxFile {
        let scene_json_file = VMaxSceneJsonFile {
            objects: vec![object("a", "contents.vmaxb", "palette.png")],
            v: 4,
            ..Default::default()
        };

        let contents = encode_contents_vmaxb_file_from_voxels(
            &[
                VMaxVoxel {
                    position: [1, 2, 3],
                    material_idx: 1,
                    color_idx: 5,
                },
                VMaxVoxel {
                    position: [40, 5, 9],
                    material_idx: 2,
                    color_idx: 7,
                },
            ],
            "uuid-a",
        );
        let settings = VMaxPaletteSettingsVmaxpsbFile {
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

        // The optional file kinds the scene graph never references:
        // enumeration, not reference-following, must find and preserve each of
        // them. History now round-trips as typed plist; selection stays
        // verbatim bytes.
        let mut history_vmaxhb_files = BTreeMap::new();
        history_vmaxhb_files.insert(
            "history.vmaxhb".to_owned(),
            VMaxHistoryVmaxhbFile {
                sessions: vec![VMaxHistorySession {
                    sid: 0,
                    steps: Vec::new(),
                    snapshots: Vec::new(),
                    ssnapshots: vec![VMaxValue::String("scene-snap".to_owned())],
                    osnapshots: Vec::new(),
                }],
                asid: 0,
            },
        );
        history_vmaxhb_files.insert(
            "scene.vmaxhb".to_owned(),
            VMaxHistoryVmaxhbFile {
                sessions: Vec::new(),
                asid: 9,
            },
        );
        let mut history_vmaxhvsb_files = BTreeMap::new();
        history_vmaxhvsb_files.insert(
            "history.vmaxhvsb".to_owned(),
            VMaxHistoryVmaxhvsbFile(Vec::new()),
        );
        let mut history_vmaxhvsc_files = BTreeMap::new();
        history_vmaxhvsc_files.insert(
            "history.vmaxhvsc".to_owned(),
            VMaxHistoryVmaxhvscFile(Vec::new()),
        );
        let mut selection_vmaxb_files = BTreeMap::new();
        selection_vmaxb_files.insert(
            "contents.selection.vmaxb".to_owned(),
            VMaxSelectionVmaxbFile(vec![10, 11, 12]),
        );

        // One of each QuickLook role: package thumbnail, per-object, per-group.
        let mut contents_vmax_pngs = BTreeMap::new();
        contents_vmax_pngs.insert("contents.vmaxb".to_owned(), image());
        let mut group_pngs = BTreeMap::new();
        group_pngs.insert("group-id".to_owned(), image());

        VMaxFile {
            scene_json_file,
            contents_files,
            palette_settings_files,
            palette_png_files,
            history_vmaxhb_files,
            history_vmaxhvsb_files,
            history_vmaxhvsc_files,
            selection_vmaxb_files,
            thumbnail_png: Some(image()),
            contents_vmax_pngs,
            group_pngs,
        }
    }

    #[test]
    fn round_trips_through_a_directory_map() {
        let file = sample();

        // `to_vmax_package` writes into an in-memory directory.
        let mut dir: HashMap<String, Vec<u8>> = HashMap::new();
        to_vmax_package(&DependenciesImpl, &file, |name, bytes| {
            dir.insert(name.to_owned(), bytes.to_vec());
            Ok(())
        })
        .unwrap();

        // Every kind, including the history / selection / QuickLook files no
        // scene object names, is written and read back through the same map.
        let read = from_vmax_package(
            &DependenciesImpl,
            || Ok(dir.keys().cloned().collect()),
            |name| Ok(dir.get(name).cloned()),
        )
        .unwrap();
        assert_eq!(read, file);

        // The three QuickLook roles land in their named maps under the right
        // keys.
        assert!(read.thumbnail_png.is_some());
        assert!(read.contents_vmax_pngs.contains_key("contents.vmaxb"));
        assert!(read.group_pngs.contains_key("group-id"));
        assert!(dir.contains_key("QuickLook/Thumbnail.png"));
        assert!(dir.contains_key("QuickLook/contents.vmaxb.png"));
        assert!(dir.contains_key("QuickLook/group-id.png"));
    }
}
