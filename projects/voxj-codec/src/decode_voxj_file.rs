use crate::{decode_voxj_object, voxj_palette_cell_counts};
use std::io;
use voxj::{VoxjCodecFile, VoxjCodecMain, VoxjSerdeFile};

/// Decodes a [`VoxjSerdeFile`] (encoded `.voxj` blocks) into a [`VoxjCodecFile`]
/// (decoded geometry), the inverse of [`encode_voxj_file`](crate::encode_voxj_file). Each
/// object's palette cell counts come from the palettes it references; the
/// palettes, hierarchy, roots, and `ext` carry over unchanged.
pub fn decode_voxj_file(file: &VoxjSerdeFile) -> io::Result<VoxjCodecFile> {
    let palettes = &file.main.palettes;
    let objects = file
        .main
        .objects
        .iter()
        .map(|object| {
            let cell_counts = voxj_palette_cell_counts(&object.palette_refs, palettes)?;
            decode_voxj_object(object, &cell_counts)
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(VoxjCodecFile {
        version: file.version,
        main: VoxjCodecMain {
            objects,
            palettes: palettes.clone(),
            hierarchy_nodes: file.main.hierarchy_nodes.clone(),
            root_hierarchy_nodes: file.main.root_hierarchy_nodes.clone(),
            ext: file.main.ext.clone(),
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        PositionEncoding, SampleEncoding, decode_voxj_file, encode_voxj_file,
        encode_voxj_file_smallest,
    };
    use std::collections::BTreeSet;
    use voxj::{VoxjCodecFile, VoxjCodecMain, VoxjCodecObject, VoxjPalette, VoxjValue};

    /// `n` single-attribute palette cells; only the count matters to the codec
    /// (it fixes each channel's packed width).
    fn cells(n: usize) -> Vec<Vec<VoxjValue>> {
        (0..n).map(|i| vec![VoxjValue::Number(i as f64)]).collect()
    }

    /// Two objects sampling two palettes of differing cell counts, with an `ext`
    /// blob, so a round trip exercises per-object geometry plus the shared
    /// palettes/ext carry-over.
    fn codec_file() -> VoxjCodecFile {
        VoxjCodecFile {
            version: 1,
            main: VoxjCodecMain {
                objects: vec![
                    VoxjCodecObject {
                        name: "a".to_owned(),
                        palette_refs: vec![0],
                        bounds: [4, 4, 4],
                        positions: vec![[0, 0, 0], [3, 1, 2], [1, 3, 0], [2, 2, 3]],
                        samples: vec![vec![1], vec![0], vec![5], vec![2]],
                    },
                    VoxjCodecObject {
                        name: "b".to_owned(),
                        palette_refs: vec![0, 1],
                        bounds: [2, 1, 1],
                        positions: vec![[0, 0, 0], [1, 0, 0]],
                        samples: vec![vec![3, 0], vec![1, 1]],
                    },
                ],
                palettes: vec![
                    VoxjPalette {
                        attributes: vec!["rgba".to_owned()],
                        data: cells(6),
                    },
                    VoxjPalette {
                        attributes: vec!["metallic".to_owned()],
                        data: cells(2),
                    },
                ],
                hierarchy_nodes: Vec::new(),
                root_hierarchy_nodes: Vec::new(),
                ext: Some(VoxjValue::Text("vendor".to_owned())),
            },
        }
    }

    /// `(position, samples)` pairs of one object, order-independent, since the
    /// block encodings reorder voxels.
    fn voxels(object: &VoxjCodecObject) -> BTreeSet<([u32; 3], Vec<u32>)> {
        object
            .positions
            .iter()
            .copied()
            .zip(object.samples.iter().cloned())
            .collect()
    }

    fn assert_round_trip(decoded: &VoxjCodecFile) {
        let original = codec_file();
        assert_eq!(decoded.version, original.version);
        assert_eq!(decoded.main.palettes, original.main.palettes);
        assert_eq!(decoded.main.ext, original.main.ext);
        assert_eq!(decoded.main.objects.len(), original.main.objects.len());
        for (got, want) in decoded.main.objects.iter().zip(&original.main.objects) {
            assert_eq!(got.name, want.name);
            assert_eq!(got.palette_refs, want.palette_refs);
            assert_eq!(got.bounds, want.bounds);
            assert_eq!(voxels(got), voxels(want));
        }
    }

    #[test]
    fn round_trips_file_through_fixed_encoding() {
        let encoded = encode_voxj_file(
            &codec_file(),
            PositionEncoding::BitmapBase64,
            SampleEncoding::PackedBase64,
        )
        .unwrap();
        assert_round_trip(&decode_voxj_file(&encoded).unwrap());
    }

    #[test]
    fn round_trips_file_through_smallest() {
        let encoded = encode_voxj_file_smallest(&codec_file()).unwrap();
        assert_round_trip(&decode_voxj_file(&encoded).unwrap());
    }
}
