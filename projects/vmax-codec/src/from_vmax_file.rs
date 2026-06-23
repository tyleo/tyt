use crate::{
    from_contents_vmaxb_file_bytes, from_history_vmaxhb_file_bytes,
    from_history_vmaxhvsb_file_bytes, from_history_vmaxhvsc_file_bytes,
    from_palette_png_file_bytes, from_palette_settings_vmaxpsb_file_bytes,
    from_quick_look_png_file_bytes, from_scene_json_file_bytes, from_selection_vmaxb_file_bytes,
};
use std::{
    collections::BTreeMap,
    io::{Error as IOError, ErrorKind, Result},
};
use vmax::VMaxSerdeFile;

/// Reads a whole `.vmax` package into a [`VMaxSerdeFile`]. `list` returns the
/// package-relative path of every file in the package (so `QuickLook/` entries
/// keep their subdirectory prefix) and `resolve` returns a file's bytes by that
/// path (`Ok(None)` when it has since vanished); together they let `from_vmax_file`
/// work without a filesystem of its own — the caller supplies the directory access.
///
/// `scene.json` is required. Every other listed file is classified by name into
/// the matching part of the package — `contents*.vmaxb`, `palette*.png`,
/// `palette*.settings.vmaxpsb`, the undo history
/// (`*.vmaxhb` / `*.vmaxhvsb` / `*.vmaxhvsc`), `*.selection.vmaxb`, and
/// `QuickLook/*` — so nothing the package holds is dropped. A file whose name
/// matches no known kind is an error rather than a silent loss.
pub fn from_vmax_file<L, R>(list: L, mut resolve: R) -> Result<VMaxSerdeFile>
where
    L: FnOnce() -> Result<Vec<String>>,
    R: FnMut(&str) -> Result<Option<Vec<u8>>>,
{
    let scene_bytes = resolve("scene.json")?
        .ok_or_else(|| IOError::new(ErrorKind::NotFound, "scene.json is missing"))?;
    let scene_json_file = from_scene_json_file_bytes(&scene_bytes)?;

    let mut contents_files = BTreeMap::new();
    let mut palette_settings_files = BTreeMap::new();
    let mut palette_png_files = BTreeMap::new();
    let mut history_vmaxhb_files = BTreeMap::new();
    let mut history_vmaxhvsb_files = BTreeMap::new();
    let mut history_vmaxhvsc_files = BTreeMap::new();
    let mut selection_vmaxb_files = BTreeMap::new();
    let mut quick_look_png_files = BTreeMap::new();

    // Classify every listed file by name and parse it into the matching map.
    // `scene.json` is already parsed above; `QuickLook/` thumbnails are matched by
    // their path prefix, and `.selection.vmaxb` is checked before `.vmaxb` so a
    // selection sidecar is not mistaken for an object. Every kind Voxel Max writes
    // is modeled, so an unrecognized name is reported rather than dropped.
    for path in list()? {
        if path == "scene.json" {
            continue;
        }
        let Some(bytes) = resolve(&path)? else {
            continue;
        };
        if path.starts_with("QuickLook/") {
            quick_look_png_files.insert(path, from_quick_look_png_file_bytes(&bytes)?);
        } else if path.ends_with(".selection.vmaxb") {
            selection_vmaxb_files.insert(path, from_selection_vmaxb_file_bytes(&bytes)?);
        } else if path.ends_with(".vmaxb") {
            contents_files.insert(path, from_contents_vmaxb_file_bytes(&bytes)?);
        } else if path.ends_with(".settings.vmaxpsb") {
            palette_settings_files.insert(path, from_palette_settings_vmaxpsb_file_bytes(&bytes)?);
        } else if path.ends_with(".png") {
            palette_png_files.insert(path, from_palette_png_file_bytes(&bytes)?);
        } else if path.ends_with(".vmaxhb") {
            history_vmaxhb_files.insert(path, from_history_vmaxhb_file_bytes(&bytes)?);
        } else if path.ends_with(".vmaxhvsb") {
            history_vmaxhvsb_files.insert(path, from_history_vmaxhvsb_file_bytes(&bytes)?);
        } else if path.ends_with(".vmaxhvsc") {
            history_vmaxhvsc_files.insert(path, from_history_vmaxhvsc_file_bytes(&bytes)?);
        } else {
            return Err(IOError::new(
                ErrorKind::InvalidData,
                format!("unrecognized file in .vmax package: {path}"),
            ));
        }
    }

    Ok(VMaxSerdeFile {
        scene_json_file,
        contents_files,
        palette_settings_files,
        palette_png_files,
        history_vmaxhb_files,
        history_vmaxhvsb_files,
        history_vmaxhvsc_files,
        selection_vmaxb_files,
        quick_look_png_files,
    })
}

#[cfg(test)]
mod tests {
    use super::from_vmax_file;
    use crate::{encode_contents_vmaxb_file_from_voxels, to_vmax_file};
    use std::collections::{BTreeMap, HashMap};
    use vmax::{
        VMaxCodecVoxel, VMaxHistoryVmaxhbFile, VMaxHistoryVmaxhvsbFile, VMaxHistoryVmaxhvscFile,
        VMaxObject, VMaxPalettePngFile, VMaxQuickLookPngFile, VMaxSceneJsonFile,
        VMaxSelectionVmaxbFile, VMaxSerdeFile, VMaxSerdePaletteSettingsVmaxpsbFile,
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

        let contents = encode_contents_vmaxb_file_from_voxels(
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

        // The optional file kinds the scene graph never references: enumeration,
        // not reference-following, must find and preserve each of them verbatim.
        let mut history_vmaxhb_files = BTreeMap::new();
        history_vmaxhb_files.insert(
            "history.vmaxhb".to_owned(),
            VMaxHistoryVmaxhbFile(vec![0x62, 0x76, 0x78, 0x32, 1, 2, 3]),
        );
        history_vmaxhb_files.insert(
            "scene.vmaxhb".to_owned(),
            VMaxHistoryVmaxhbFile(vec![0x62, 0x76, 0x78, 0x32, 9]),
        );
        let mut history_vmaxhvsb_files = BTreeMap::new();
        history_vmaxhvsb_files.insert(
            "history.vmaxhvsb".to_owned(),
            VMaxHistoryVmaxhvsbFile(vec![0x62, 0x76, 0x78, 0x32, 4, 5]),
        );
        let mut history_vmaxhvsc_files = BTreeMap::new();
        history_vmaxhvsc_files.insert(
            "history.vmaxhvsc".to_owned(),
            VMaxHistoryVmaxhvscFile(vec![0x62, 0x70, 0x6c, 0x69, 0x73, 0x74]),
        );
        let mut selection_vmaxb_files = BTreeMap::new();
        selection_vmaxb_files.insert(
            "contents.selection.vmaxb".to_owned(),
            VMaxSelectionVmaxbFile(vec![10, 11, 12]),
        );
        let mut quick_look_png_files = BTreeMap::new();
        quick_look_png_files.insert(
            "QuickLook/Thumbnail.png".to_owned(),
            VMaxQuickLookPngFile(vec![0x89, b'P', b'N', b'G', 1, 2]),
        );
        quick_look_png_files.insert(
            "QuickLook/contents.vmaxb.png".to_owned(),
            VMaxQuickLookPngFile(vec![0x89, b'P', b'N', b'G', 7, 8]),
        );

        VMaxSerdeFile {
            scene_json_file,
            contents_files,
            palette_settings_files,
            palette_png_files,
            history_vmaxhb_files,
            history_vmaxhvsb_files,
            history_vmaxhvsc_files,
            selection_vmaxb_files,
            quick_look_png_files,
        }
    }

    #[test]
    fn round_trips_through_a_directory_map() {
        let file = sample();

        // `to_vmax_file` writes into an in-memory directory.
        let mut dir: HashMap<String, Vec<u8>> = HashMap::new();
        to_vmax_file(&file, |name, bytes| {
            dir.insert(name.to_owned(), bytes.to_vec());
            Ok(())
        })
        .unwrap();

        // Every kind, including the history / selection / QuickLook files no scene
        // object names, is written and read back through the same map.
        let read = from_vmax_file(
            || Ok(dir.keys().cloned().collect()),
            |name| Ok(dir.get(name).cloned()),
        )
        .unwrap();
        assert_eq!(read, file);
    }
}
