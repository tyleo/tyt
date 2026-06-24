use crate::{Result, decode_contents_vmaxb_file, decode_palette_settings_vmaxpsb_file};
use std::collections::BTreeMap;
use vmax::{VMaxCodecFile, VMaxSerdeFile};

/// Decodes a [`VMaxSerdeFile`] into a [`VMaxCodecFile`], the inverse of
/// [`encode_vmax_file`](crate::encode_vmax_file).
pub fn decode_vmax_file(file: &VMaxSerdeFile) -> Result<VMaxCodecFile> {
    let contents_files = file
        .contents_files
        .iter()
        .map(|(name, contents)| Ok((name.clone(), decode_contents_vmaxb_file(contents)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    let palette_settings_files = file
        .palette_settings_files
        .iter()
        .map(|(name, palette)| Ok((name.clone(), decode_palette_settings_vmaxpsb_file(palette)?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(VMaxCodecFile {
        scene_json_file: file.scene_json_file.clone(),
        contents_files,
        palette_settings_files,
        palette_png_files: file.palette_png_files.clone(),
        history_vmaxhb_files: file.history_vmaxhb_files.clone(),
        history_vmaxhvsb_files: file.history_vmaxhvsb_files.clone(),
        history_vmaxhvsc_files: file.history_vmaxhvsc_files.clone(),
        selection_vmaxb_files: file.selection_vmaxb_files.clone(),
        quick_look_png_files: file.quick_look_png_files.clone(),
    })
}

#[cfg(test)]
mod tests {
    use crate::{decode_vmax_file, encode_vmax_file, encode_vmax_snapshots};
    use std::collections::BTreeMap;
    use vmax::{
        VMaxBrush, VMaxCamera, VMaxCodecVoxel, VMaxHistoryVmaxhbFile, VMaxMaterialDispersion,
        VMaxPalettePngFile, VMaxQuickLookPngFile, VMaxSceneJsonFile, VMaxSerdeContentsVmaxbFile,
        VMaxSerdeFile, VMaxSerdeMaterial, VMaxSerdePaletteSettingsVmaxpsbFile, VMaxTools,
    };

    fn voxel(x: i32, y: i32, z: i32, material_idx: u8, color_idx: u8) -> VMaxCodecVoxel {
        VMaxCodecVoxel {
            position: [x, y, z],
            material_idx,
            color_idx,
        }
    }

    /// A raw serde package already in the canonical form `encode_vmax_file`
    /// produces: one checkpoint snapshot per chunk (so the edit-log collapse is
    /// a no-op), materials whose `mi` runs `1..N` in slot order, packed colors,
    /// and `Some` editor state on the object. Decoding then re-encoding it must
    /// reproduce it exactly, which pins the lossless promise across snapshot
    /// geometry, `mi` reconstruction, color packing, and every preserved
    /// editor-state field.
    fn serde_file() -> VMaxSerdeFile {
        let contents = VMaxSerdeContentsVmaxbFile {
            // color_idx != 0 (0 is the format's empty marker); spans three chunks.
            snapshots: encode_vmax_snapshots(&[
                voxel(0, 0, 0, 0, 7),
                voxel(5, 5, 5, 7, 1),
                voxel(40, 5, 9, 2, 200),
                voxel(70, 0, 0, 1, 5),
            ]),
            uuid: "uuid-a".to_owned(),
            v: 4,
            tools: Some(VMaxTools {
                bs: 5,
                ..Default::default()
            }),
            brush: Some(VMaxBrush {
                name: "brush".to_owned(),
                current: 2,
                ..Default::default()
            }),
            cam: Some(VMaxCamera {
                wa: 1.0,
                ha: 2.0,
                da: 3.0,
                lwa: 4.0,
                lha: 5.0,
                lda: 6.0,
                px: 7.0,
                py: 8.0,
                z: 9.0,
                o: [10.0, 11.0, 12.0],
            }),
        };
        let palette = VMaxSerdePaletteSettingsVmaxpsbFile {
            name: "pal".to_owned(),
            // `mi` in slot order so encode's slot-derived `mi` reproduces it.
            materials: vec![
                VMaxSerdeMaterial {
                    mi: "1".to_owned(),
                    mc: 0.66,
                    rc: 0.58,
                    sic: 4.2,
                    sh: true,
                    md: Some(VMaxMaterialDispersion {
                        absorption: 0.0,
                        ior: 1.5,
                        transmission: 0.83,
                    }),
                },
                VMaxSerdeMaterial {
                    mi: "2".to_owned(),
                    mc: 0.1,
                    rc: 0.9,
                    sic: 0.0,
                    sh: false,
                    md: None,
                },
            ],
            colors: vec![1, 2, 3, 255, 4, 5, 6, 0],
            indices: vec![1, 2, 3],
            lc: vec![0u8; 256],
            palette_type: 1,
            transparency: 0.5,
            r: 2,
            rt: "n".to_owned(),
            cmt: "ng".to_owned(),
            current: 3,
            ali: "1".to_owned(),
        };

        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents.vmaxb".to_owned(), contents);
        let mut palette_settings_files = BTreeMap::new();
        palette_settings_files.insert("palette.settings.vmaxpsb".to_owned(), palette);
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert(
            "palette.png".to_owned(),
            VMaxPalettePngFile(vec![[1, 2, 3, 255], [4, 5, 6, 0]]),
        );
        // The opaque preserved file kinds (history + a QuickLook thumbnail):
        // decoding then re-encoding must carry them through byte for byte.
        let mut history_vmaxhb_files = BTreeMap::new();
        history_vmaxhb_files.insert(
            "history.vmaxhb".to_owned(),
            VMaxHistoryVmaxhbFile(vec![0x62, 0x76, 0x78, 0x32, 9]),
        );
        let mut quick_look_png_files = BTreeMap::new();
        quick_look_png_files.insert(
            "QuickLook/Thumbnail.png".to_owned(),
            VMaxQuickLookPngFile(vec![0x89, b'P', 1, 2]),
        );

        VMaxSerdeFile {
            scene_json_file: VMaxSceneJsonFile {
                v: 4,
                ..Default::default()
            },
            contents_files,
            palette_settings_files,
            palette_png_files,
            history_vmaxhb_files,
            history_vmaxhvsb_files: BTreeMap::new(),
            history_vmaxhvsc_files: BTreeMap::new(),
            selection_vmaxb_files: BTreeMap::new(),
            quick_look_png_files,
        }
    }

    /// `decode_vmax_file` then `encode_vmax_file` reproduces the original serde
    /// package: the promised lossless round trip (the snapshots are already
    /// canonical, so the edit-log collapse is a no-op here).
    #[test]
    fn round_trips_serde_file_through_codec() {
        let original = serde_file();
        let decoded = decode_vmax_file(&original).unwrap();
        // Decode actually recovered the voxels, not an empty/degenerate object.
        assert_eq!(decoded.contents_files["contents.vmaxb"].voxels.len(), 4);
        assert_eq!(encode_vmax_file(&decoded), original);
    }
}
