use crate::{
    ColorFormat, EditStateMode, Result, voxj_decoded_object_from_vox_object,
    voxj_hierarchy_node_from_vox_hierarchy_node, voxj_palette_from_vox_palette,
    voxj_value_from_vox_value, voxj_value_pool_from_vox_value_pool,
};
use ty_math::TyVector3U32;
use voxcore::{VoxMain, VoxObject};
use voxj::{VoxjEditObject, VoxjEditState, VoxjFile, VoxjMain, VoxjRuntimeState};
use voxj_codec::{
    PositionEncoding, SampleEncoding, encode_voxj_object_optimized, voxj_palette_material_counts,
};

/// The voxj format version stamped on documents written from a [`VoxMain`],
/// which does not itself carry a version.
const VOXJ_FORMAT_VERSION: u32 = 1;

/// Builds a [`VoxjFile`] from a [`VoxMain`], the workhorse behind
/// [`to_voxj_file`](crate::to_voxj_file) and
/// [`VoxjFileBuilder`](crate::VoxjFileBuilder).
///
/// Value pools, palettes, objects, and hierarchy nodes are emitted in id order,
/// so each lands at its original array index and the cross references carry
/// over unchanged.
///
/// # Arguments
/// * `state` - the voxel model to encode.
/// * `position` - position-block encoding, or `None` to search for the
///   smallest.
/// * `sample` - sample-block encoding, or `None` to search for the smallest.
/// * `ext` - when false, omits the user-defined `ext` extension block.
/// * `edit_state` - when to record each object's editor build volume.
/// * `color_format` - the on-wire encoding for sRGB color pools.
pub fn write_voxj(
    state: &VoxMain,
    position: Option<PositionEncoding>,
    sample: Option<SampleEncoding>,
    ext: bool,
    edit_state: EditStateMode,
    color_format: ColorFormat,
) -> Result<VoxjFile> {
    let value_pools = state
        .iter_value_pools()
        .map(|(_, pool)| voxj_value_pool_from_vox_value_pool(pool, color_format))
        .collect::<Vec<_>>();

    let palettes = state
        .iter_palettes()
        .map(|(_, palette)| voxj_palette_from_vox_palette(palette))
        .collect::<Vec<_>>();

    let objects = state
        .iter_objects()
        .map(|(object_id, _)| {
            let decoded = voxj_decoded_object_from_vox_object(state, object_id);
            let material_counts = voxj_palette_material_counts(&decoded.layers, &palettes)?;
            encode_voxj_object_optimized(&decoded, &material_counts, position, sample)
        })
        .collect::<voxj_codec::Result<Vec<_>>>()?;

    let nodes = state
        .iter_hierarchy_nodes()
        .map(|(_, node)| voxj_hierarchy_node_from_vox_hierarchy_node(node))
        .collect();

    let root_nodes = state
        .root_hierarchy_nodes()
        .iter()
        .map(|id| id.to_u32() as usize)
        .collect();

    // Editor state, aligned by index with the objects. Each entry is the
    // object's build volume, recorded when `edit_state` calls for it.
    let edit_state = emit_edit_state(state, edit_state).then(|| VoxjEditState {
        objects: state
            .iter_objects()
            .map(|(_, object)| {
                let bounds = object.bounds();
                let origin = object.origin();
                VoxjEditObject {
                    bounds: bounds.to_array(),
                    origin: origin.to_array(),
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
                value_pools,
                palettes,
                objects,
                nodes,
                root_nodes,
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

/// Whether an object's build volume already equals its tight runtime grid, so
/// it needs no edit-state entry to be recovered on load.
fn is_tight(object: &VoxObject) -> bool {
    let bounds = object.bounds();
    match object.live_extent() {
        Some((min, size)) => min == TyVector3U32::default() && size == bounds,
        None => bounds == TyVector3U32::default(),
    }
}
