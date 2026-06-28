use crate::{
    Result, voxj_decoded_object_from_vox_object, voxj_hierarchy_node_from_vox_hierarchy_node,
    voxj_palette_from_vox_palette, voxj_value_from_vox_value,
};
use voxcore::{VoxEditObject, VoxState};
use voxj::{VoxjEditObject, VoxjEditState, VoxjFile, VoxjMain, VoxjRuntimeState};
use voxj_codec::{
    PositionEncoding, SampleEncoding, encode_voxj_object, encode_voxj_object_smallest,
    voxj_palette_cell_counts,
};

/// The voxj format version stamped on documents written from a [`VoxState`],
/// which does not itself carry a version.
pub(crate) const VOXJ_FORMAT_VERSION: u32 = 1;

/// Builds a [`VoxjFile`] from a [`VoxState`], encoding each object's geometry
/// with `encoding` (a fixed position/sample pair) or, when `None`, the smallest
/// per-object block encodings. The shared step behind the `.voxj` and `.voxjz`
/// writers and the fixed/smallest entry points. Objects, palettes, and hierarchy
/// nodes are emitted in id order, so each lands at its original array index and
/// the cross references carry over unchanged.
pub(crate) fn to_voxj_file_with_encoding(
    state: &VoxState,
    encoding: Option<(PositionEncoding, SampleEncoding)>,
) -> Result<VoxjFile> {
    let palettes = state
        .iter_palettes()
        .map(|(_, palette)| voxj_palette_from_vox_palette(palette))
        .collect::<Vec<_>>();

    let objects = state
        .iter_objects()
        .map(|(_, object)| {
            let decoded = voxj_decoded_object_from_vox_object(object);
            let cell_counts = voxj_palette_cell_counts(&decoded.palette_refs, &palettes)?;
            match encoding {
                Some((position, sample)) => {
                    encode_voxj_object(&decoded, &cell_counts, position, sample)
                }
                None => encode_voxj_object_smallest(&decoded, &cell_counts),
            }
        })
        .collect::<voxj_codec::Result<Vec<_>>>()?;

    let hierarchy_nodes = state
        .iter_hierarchy_nodes()
        .map(|(_, node)| voxj_hierarchy_node_from_vox_hierarchy_node(node))
        .collect();

    let root_hierarchy_nodes = state
        .root_hierarchy_nodes()
        .iter()
        .map(|id| id.to_u32() as usize)
        .collect();

    // Editor state, aligned by index with the objects. Emitted only when some
    // object's edit grid differs from its runtime grid; the zero-margin default
    // is recreated by `from_voxj_file`, so omitting it loses nothing.
    let any_margin = state.iter_objects().any(|(id, object)| {
        let runtime = VoxEditObject {
            bounds: object.bounds(),
            origin: object.origin(),
        };
        state.edit_object(id) != Some(runtime)
    });
    let edit_state = any_margin.then(|| VoxjEditState {
        objects: state
            .iter_objects()
            .map(|(id, _)| {
                let edit = state
                    .edit_object(id)
                    .expect("a retained object has an edit grid");
                VoxjEditObject {
                    bounds: [edit.bounds.x, edit.bounds.y, edit.bounds.z],
                    origin: [edit.origin.x, edit.origin.y, edit.origin.z],
                }
            })
            .collect(),
    });

    let ext = state.ext().map(voxj_value_from_vox_value);

    Ok(VoxjFile {
        version: VOXJ_FORMAT_VERSION,
        main: VoxjMain {
            runtime_state: VoxjRuntimeState {
                objects,
                palettes,
                hierarchy_nodes,
                root_hierarchy_nodes,
            },
            edit_state,
            ext,
        },
    })
}
