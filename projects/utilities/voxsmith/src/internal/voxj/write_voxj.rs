use crate::{
    EditStateMode, Result, voxj_decoded_object_from_vox_object,
    voxj_hierarchy_node_from_vox_hierarchy_node, voxj_palette_from_vox_palette,
    voxj_value_from_vox_value,
};
use ty_math::TyVector3U32;
use voxcore::{VoxMain, VoxObject};
use voxj::{VoxjEditObject, VoxjEditState, VoxjFile, VoxjMain, VoxjRuntimeState};
use voxj_codec::{
    PositionEncoding, SampleEncoding, encode_voxj_object, encode_voxj_object_smallest,
    voxj_palette_cell_counts,
};

/// The voxj format version stamped on documents written from a [`VoxMain`],
/// which does not itself carry a version.
const VOXJ_FORMAT_VERSION: u32 = 1;

/// Builds a [`VoxjFile`] from a [`VoxMain`], the workhorse behind
/// [`to_voxj_file`](crate::to_voxj_file) and
/// [`VoxjFileBuilder`](crate::VoxjFileBuilder).
///
/// Each object's geometry is encoded with `encoding`, a fixed position/sample
/// pair, or the smallest per-object block encodings when `None`. When `ext` is
/// false the user-defined ext block is dropped. `edit_state` selects when each
/// object's build volume is recorded. Objects, palettes, and hierarchy nodes are
/// emitted in id order, so each lands at its original array index and the cross
/// references carry over unchanged.
pub(crate) fn write_voxj(
    state: &VoxMain,
    encoding: Option<(PositionEncoding, SampleEncoding)>,
    ext: bool,
    edit_state: EditStateMode,
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

    // Editor state, aligned by index with the objects. Each entry is the object's
    // build volume, recorded when `edit_state` calls for it.
    let edit_state = emit_edit_state(state, edit_state).then(|| VoxjEditState {
        objects: state
            .iter_objects()
            .map(|(_, object)| {
                let bounds = object.bounds();
                let origin = object.origin();
                VoxjEditObject {
                    bounds: [bounds.x, bounds.y, bounds.z],
                    origin: [origin.x, origin.y, origin.z],
                }
            })
            .collect(),
    });

    let ext = if ext {
        state.ext().map(voxj_value_from_vox_value)
    } else {
        None
    };

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

/// Whether the document records editor build volumes under `mode`. Auto records
/// them only when some object carries margin around its live voxels, since an
/// already-tight object recreates its build volume on load.
fn emit_edit_state(state: &VoxMain, mode: EditStateMode) -> bool {
    match mode {
        EditStateMode::Always => true,
        EditStateMode::Never => false,
        EditStateMode::Auto => state.iter_objects().any(|(_, object)| !is_tight(object)),
    }
}

/// Whether an object's build volume already equals its tight runtime grid, so it
/// needs no edit-state entry to be recovered on load.
fn is_tight(object: &VoxObject) -> bool {
    let bounds = object.bounds();
    match object.live_extent() {
        Some((min, size)) => min == TyVector3U32::default() && size == bounds,
        None => bounds == TyVector3U32::default(),
    }
}
