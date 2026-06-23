use crate::{encode_voxj_object_smallest, voxj_palette_cell_counts};
use std::io;
use voxj::{VoxjCodecFile, VoxjSerdeFile, VoxjSerdeMain};

/// Encodes a [`VoxjCodecFile`] (decoded geometry) into a [`VoxjSerdeFile`]
/// (encoded `.voxj` blocks), choosing each object's block encodings by the
/// smallest-deflated search ([`encode_voxj_object_smallest`](crate::encode_voxj_object_smallest)).
/// The canonical shipping form. Each object's palette cell counts come from the
/// palettes it references; the palettes, hierarchy, roots, and `ext` carry over
/// unchanged. Errors if an object references a palette outside the document's
/// `palettes`.
pub fn encode_voxj_file_smallest(file: &VoxjCodecFile) -> io::Result<VoxjSerdeFile> {
    let palettes = &file.main.palettes;
    let objects = file
        .main
        .objects
        .iter()
        .map(|object| {
            let cell_counts = voxj_palette_cell_counts(&object.palette_refs, palettes)?;
            encode_voxj_object_smallest(object, &cell_counts)
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(VoxjSerdeFile {
        version: file.version,
        main: VoxjSerdeMain {
            objects,
            palettes: palettes.clone(),
            hierarchy_nodes: file.main.hierarchy_nodes.clone(),
            root_hierarchy_nodes: file.main.root_hierarchy_nodes.clone(),
            ext: file.main.ext.clone(),
        },
    })
}
