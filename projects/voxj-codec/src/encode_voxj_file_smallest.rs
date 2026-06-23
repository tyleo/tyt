use crate::{encode_voxj_object_smallest, voxj_palette_cell_counts};
use voxj::{VoxjCodecFile, VoxjSerdeFile, VoxjSerdeMain};

/// Encodes a [`VoxjCodecFile`] (decoded geometry) into a [`VoxjSerdeFile`]
/// (encoded `.voxj` blocks), choosing each object's block encodings by the
/// smallest-deflated search ([`encode_voxj_object_smallest`](crate::encode_voxj_object_smallest)).
/// The canonical shipping form. Each object's palette cell counts come from the
/// palettes it references; the palettes, hierarchy, roots, and `ext` carry over
/// unchanged.
pub fn encode_voxj_file_smallest(file: &VoxjCodecFile) -> VoxjSerdeFile {
    let palettes = &file.main.palettes;
    let objects = file
        .main
        .objects
        .iter()
        .map(|object| {
            let cell_counts = voxj_palette_cell_counts(&object.palette_refs, palettes);
            encode_voxj_object_smallest(object, &cell_counts)
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
