use crate::{
    DecodePng, DecodeVMaxPlist, DecodeVMaxSceneJson, DecompressLzfse, Error, Result,
    from_contents_vmaxb_file_bytes, from_image_png_file_bytes, from_palette_png_file_bytes,
    from_palette_settings_vmaxpsb_file_bytes, from_scene_json_file_bytes,
    from_selection_vmaxb_file_bytes,
};
use std::collections::BTreeMap;
use vmax::{VMaxFile, VMaxOpaqueFile};

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
    let scene_json_file = from_scene_json_file_bytes(dependencies, &scene_bytes)
        .map_err(|error| Error::File("scene.json".to_owned(), Box::new(error)))?;

    let mut contents_files = BTreeMap::new();
    let mut palette_settings_files = BTreeMap::new();
    let mut palette_png_files = BTreeMap::new();
    let mut selection_vmaxb_files = BTreeMap::new();
    let mut thumbnail_png = None;
    let mut contents_vmax_pngs = BTreeMap::new();
    let mut group_pngs = BTreeMap::new();
    let mut other_files = BTreeMap::new();

    // Classify every listed file by name and parse it into the matching map.
    // `scene.json` is already parsed above. `QuickLook/` thumbnails split by
    // role: the package `Thumbnail.png`, the per-object `contents*.vmaxb.png`,
    // and the per-group `<id>.png`. `.selection.vmaxb` is checked before
    // `.vmaxb` so a selection sidecar is not mistaken for an object. The undo
    // history is skipped unread, since Voxel Max opens a package without it,
    // and every other file is kept as stored, as Voxel Max keeps a file it
    // does not know.
    for path in list()? {
        if path == "scene.json" || is_history(&path) {
            continue;
        }
        let Some(bytes) = resolve(&path)? else {
            continue;
        };
        let in_file = |error: Error| Error::File(path.clone(), Box::new(error));
        if let Some(name) = path.strip_prefix(QUICK_LOOK_PREFIX) {
            if path == THUMBNAIL_PATH {
                thumbnail_png =
                    Some(from_image_png_file_bytes(dependencies, &bytes).map_err(in_file)?);
            } else if let Some(data) = name.strip_suffix(".png").and_then(strip_contents_suffix) {
                let image = from_image_png_file_bytes(dependencies, &bytes).map_err(in_file)?;
                contents_vmax_pngs.insert(data, image);
            } else if let Some(id) = name.strip_suffix(".png") {
                let image = from_image_png_file_bytes(dependencies, &bytes).map_err(in_file)?;
                group_pngs.insert(id.to_owned(), image);
            } else {
                other_files.insert(path, VMaxOpaqueFile(bytes));
            }
        } else if path.ends_with(".selection.vmaxb") {
            let selection = from_selection_vmaxb_file_bytes(&bytes).map_err(in_file)?;
            selection_vmaxb_files.insert(path, selection);
        } else if path.ends_with(".vmaxb") {
            let contents = from_contents_vmaxb_file_bytes(dependencies, &bytes).map_err(in_file)?;
            contents_files.insert(path, contents);
        } else if path.ends_with(".settings.vmaxpsb") {
            let settings =
                from_palette_settings_vmaxpsb_file_bytes(dependencies, &bytes).map_err(in_file)?;
            palette_settings_files.insert(path, settings);
        } else if path.ends_with(".png") {
            let palette = from_palette_png_file_bytes(dependencies, &bytes).map_err(in_file)?;
            palette_png_files.insert(path, palette);
        } else {
            other_files.insert(path, VMaxOpaqueFile(bytes));
        }
    }

    Ok(VMaxFile {
        scene_json_file,
        contents_files,
        palette_settings_files,
        palette_png_files,
        selection_vmaxb_files,
        thumbnail_png,
        contents_vmax_pngs,
        group_pngs,
        other_files,
    })
}

/// Returns the object `data` filename a per-object thumbnail names (a
/// `contents*.vmaxb` stem), or `None` when the name is not a contents preview.
fn strip_contents_suffix(name: &str) -> Option<String> {
    name.ends_with(".vmaxb").then(|| name.to_owned())
}

/// Whether `path` names an undo-history file: `*.vmaxhb` (including
/// `scene.vmaxhb`), `*.vmaxhvsb`, or `*.vmaxhvsc`.
fn is_history(path: &str) -> bool {
    [".vmaxhb", ".vmaxhvsb", ".vmaxhvsc"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{DependenciesImpl, Error, from_vmax_package, to_vmax_package};
    use std::collections::{BTreeMap, HashMap};
    use vmax::{
        VMaxFile, VMaxImage, VMaxObject, VMaxOpaqueFile, VMaxPalettePngFile,
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
            t_prp: None,
            e_cm: None,
            e_cmv: None,
            e_vc: None,
            e_vm: None,
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
        // them. Selection and every other file stay as stored.
        let mut other_files = BTreeMap::new();
        other_files.insert(
            "animations.vmaxa".to_owned(),
            VMaxOpaqueFile(br#"{"clips":[]}"#.to_vec()),
        );
        other_files.insert(
            "QuickLook/notes.txt".to_owned(),
            VMaxOpaqueFile(b"kept".to_vec()),
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
            selection_vmaxb_files,
            thumbnail_png: Some(image()),
            contents_vmax_pngs,
            group_pngs,
            other_files,
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

        // Every kind, including the selection / QuickLook / other files no
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

    /// The undo history is skipped unread, so a package whose history this
    /// crate cannot decode still opens, as Voxel Max opens one without it.
    #[test]
    fn history_files_are_skipped() {
        let mut dir: HashMap<String, Vec<u8>> = HashMap::new();
        to_vmax_package(&DependenciesImpl, &sample(), |name, bytes| {
            dir.insert(name.to_owned(), bytes.to_vec());
            Ok(())
        })
        .unwrap();
        for name in [
            "history.vmaxhb",
            "scene.vmaxhb",
            "history1.vmaxhvsb",
            "history1.vmaxhvsc",
        ] {
            dir.insert(name.to_owned(), b"not a plist".to_vec());
        }

        let read = from_vmax_package(
            &DependenciesImpl,
            || Ok(dir.keys().cloned().collect()),
            |name| Ok(dir.get(name).cloned()),
        )
        .unwrap();

        assert_eq!(read, sample());
    }

    #[test]
    fn a_scene_without_a_json_form_fails_to_write() {
        let mut nan_position = sample();
        nan_position.scene_json_file.objects[0].position[0] = f64::NAN;

        let mut infinite_setting = sample();
        infinite_setting.scene_json_file.aint = Some(f64::INFINITY);

        let mut nan_pivot_offset = sample();
        nan_pivot_offset.scene_json_file.objects[0].t_po =
            Some(VMaxValue::Array(vec![VMaxValue::Real(f64::NAN)]));

        let mut data_pivot_offset = sample();
        data_pivot_offset.scene_json_file.objects[0].t_po = Some(VMaxValue::Dictionary(
            BTreeMap::from([("x".to_owned(), VMaxValue::Data(vec![1, 2]))]),
        ));

        for file in [
            nan_position,
            infinite_setting,
            nan_pivot_offset,
            data_pivot_offset,
        ] {
            assert!(matches!(
                to_vmax_package(&DependenciesImpl, &file, |_, _| Ok(())),
                Err(Error::Json(_))
            ));
        }
    }
}
