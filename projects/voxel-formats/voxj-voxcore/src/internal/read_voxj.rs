use crate::{
    Error, Result, VoxjVoxExtSink, vox_hierarchy_node_from_voxj_hierarchy_node,
    vox_object_from_voxj_decoded_object, vox_palette_from_voxj_palette,
    vox_value_pool_from_voxj_value_pool,
};
use branded_id::U32Id;
use voxcore::VoxMain;
use voxj::{
    DecodeBase64, VoxjFile,
    objects::{decode_voxj_object, voxj_palette_material_counts},
};

/// Loads a [`VoxjFile`] into a [`VoxMain`] whose ext records what `T` keeps,
/// the body of [`from_voxj_file`](crate::from_voxj_file) and
/// [`from_voxj_file_with_ext`](crate::ext::from_voxj_file_with_ext). Each
/// object's position and sample blocks decode through `dependencies`.
/// Entities take ids in listing order, so each id equals its voxj array
/// index and the cross-references carry over. The nodes land as one batch
/// because the wire permits a node to list a child that appears later.
///
/// Errors if:
///
/// 1. a block is malformed
/// 2. object geometry is malformed
/// 3. a checked insertion rejects a cross-reference
/// 4. the ext records a malformed `ext` block
pub fn read_voxj<T: VoxjVoxExtSink, D: DecodeBase64>(
    dependencies: &D,
    file: &VoxjFile,
) -> Result<VoxMain<T>> {
    let main = &file.main;
    let mut state = VoxMain::default();

    // Build each value before adding it so a failed conversion leaves the state
    // untouched. Value pools land first, so palette properties resolve against
    // them.
    for value_pool in &main.runtime_state.value_pools {
        state.retain_value_pool(vox_value_pool_from_voxj_value_pool(value_pool)?);
    }

    // An insertion identifies the entity it rejected by the ids it holds,
    // which are internal to the palette or object; the listing index points
    // back into the document.
    for (index, palette) in main.runtime_state.palettes.iter().enumerate() {
        state
            .retain_palette(vox_palette_from_voxj_palette(palette)?)
            .map_err(|error| Error::invalid(format!("palette {index}: {error}")))?;
    }

    for (index, object) in main.runtime_state.objects.iter().enumerate() {
        let material_counts =
            voxj_palette_material_counts(&object.layers, &main.runtime_state.palettes)?;
        let decoded = decode_voxj_object(dependencies, object, &material_counts)?;
        // The build volume, present only when the document recorded margin
        // around the object's live voxels; otherwise the runtime grid is the
        // build volume.
        let edit = main
            .edit_state
            .as_ref()
            .and_then(|e| e.objects.get(index))
            .map(|edit| (edit.bounds, edit.origin));
        let vox_object = vox_object_from_voxj_decoded_object(&decoded, edit)?;
        state
            .retain_object(vox_object)
            .map_err(|error| Error::invalid(format!("object {index}: {error}")))?;
    }

    let nodes = main
        .runtime_state
        .nodes
        .iter()
        .map(vox_hierarchy_node_from_voxj_hierarchy_node)
        .collect::<Result<Vec<_>>>()?;
    state.retain_hierarchy_nodes(nodes)?;

    state.set_root_hierarchy_node_ids(
        main.runtime_state
            .root_nodes
            .iter()
            .map(|&index| U32Id::from_u32(index as u32))
            .collect(),
    )?;

    T::record_block(state, main.ext.as_ref())
}
