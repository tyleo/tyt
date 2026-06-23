use crate::{PositionEncoding, SampleEncoding, encode_voxj_object, voxj_palette_cell_counts};
use voxj::{VoxjCodecFile, VoxjSerdeFile, VoxjSerdeMain};

/// Encodes a [`VoxjCodecFile`] (decoded geometry) into a [`VoxjSerdeFile`]
/// (encoded `.voxj` blocks) with fixed position and sample encodings, ready to
/// serialize with [`to_voxj_file_bytes`](crate::to_voxj_file_bytes). Each object's palette
/// cell counts come from the palettes it references; the palettes, hierarchy,
/// roots, and `ext` carry over unchanged.
pub fn encode_voxj_file(
    file: &VoxjCodecFile,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> VoxjSerdeFile {
    let palettes = &file.main.palettes;
    let objects = file
        .main
        .objects
        .iter()
        .map(|object| {
            let cell_counts = voxj_palette_cell_counts(&object.palette_refs, palettes);
            encode_voxj_object(object, &cell_counts, position, sample)
        })
        .collect();
    VoxjSerdeFile {
        version: file.version,
        main: VoxjSerdeMain {
            objects,
            palettes: palettes.clone(),
            hierarchy_nodes: file.main.hierarchy_nodes.clone(),
            root_hierarchy_nodes: file.main.root_hierarchy_nodes.clone(),
            ext: file.main.ext.clone(),
        },
    }
}
