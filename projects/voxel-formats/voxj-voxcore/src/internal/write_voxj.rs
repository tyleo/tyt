use crate::{
    EditStateMode, Result, VoxjVoxExtSource, VoxjWriteOptions, voxj_decoded_object_from_vox_object,
    voxj_hierarchy_node_from_vox_hierarchy_node, voxj_palette_from_vox_palette,
    voxj_value_pool_from_vox_value_pool,
};
use ty_math::TyVector3U32;
use voxcore::{VoxMain, VoxObject};
use voxj::{
    CostVoxjObject, EncodeBase64, VoxjEditObject, VoxjEditState, VoxjFile, VoxjMain,
    VoxjRuntimeState,
    objects::{
        Result as ObjectsResult, encode_voxj_object_optimized, voxj_palette_material_counts,
    },
};

/// The voxj format version stamped on every written document. A [`VoxMain`]
/// carries no version.
const VOXJ_FORMAT_VERSION: u32 = 1;

/// Encodes a [`VoxMain`] into a [`VoxjFile`] with the block its ext supplies,
/// the body of [`to_voxj_file`](crate::to_voxj_file) and
/// [`to_voxj_file_with_ext`](crate::ext::to_voxj_file_with_ext).
///
/// Value pools, palettes, objects, and hierarchy nodes are emitted in listing
/// order, and ids are written as array indices. Call
/// [`VoxMain::gc`](voxcore::VoxMain::gc) first after any removal or move so
/// each id equals its listing index and the cross references land intact.
///
/// # Arguments
/// 1. `dependencies` - encodes the base64 blocks and costs a searched block's
///    candidates. A pinned pair is never costed.
pub fn write_voxj<T: VoxjVoxExtSource, D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    state: &VoxMain<T>,
    options: &VoxjWriteOptions,
) -> Result<VoxjFile> {
    let value_pools = state
        .iter_value_pools()
        .map(|(_, value_pool)| voxj_value_pool_from_vox_value_pool(value_pool))
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
            encode_voxj_object_optimized(
                dependencies,
                &decoded,
                &material_counts,
                options.position_encoding,
                options.sample_encoding,
            )
        })
        .collect::<ObjectsResult<Vec<_>>>()?;

    let nodes = state
        .iter_hierarchy_nodes()
        .map(|(_, node)| voxj_hierarchy_node_from_vox_hierarchy_node(node))
        .collect();

    let root_nodes = state
        .root_hierarchy_node_ids()
        .iter()
        .map(|node_id| node_id.to_u32() as usize)
        .collect();

    // Editor state, aligned by index with the objects. Each entry is the
    // object's build volume.
    let edit_state = emit_edit_state(state, options.edit_state).then(|| VoxjEditState {
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

    let ext = if options.ext {
        state.ext().ext_block()
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
/// them only when some object carries margin around its live voxels. An
/// already-tight object recreates its build volume on load.
fn emit_edit_state<T>(state: &VoxMain<T>, mode: EditStateMode) -> bool {
    match mode {
        EditStateMode::Always => true,
        EditStateMode::Never => false,
        EditStateMode::Auto => state.iter_objects().any(|(_, object)| !is_tight(object)),
    }
}

/// Whether an object's build volume already equals its tight runtime grid.
/// Such an object needs no edit-state entry to be recovered on load.
fn is_tight(object: &VoxObject) -> bool {
    let bounds = object.bounds();
    match object.live_extent() {
        Some((min, size)) => min == TyVector3U32::default() && size == bounds,
        None => bounds == TyVector3U32::default(),
    }
}
