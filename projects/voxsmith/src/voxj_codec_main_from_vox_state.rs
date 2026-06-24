use crate::{
    voxj_codec_object_from_vox_object, voxj_hierarchy_node_from_vox_hierarchy_node,
    voxj_palette_from_vox_palette, voxj_value_from_vox_value,
};
use voxcore::VoxState;
use voxj::VoxjCodecMain;

/// Writes a [`VoxState`] back into a [`VoxjCodecMain`] (decoded voxj geometry).
/// Objects, palettes, and hierarchy nodes are emitted in id order, so each lands
/// at its original array index and the cross references carry over unchanged.
/// Each object emits one voxel per live cell, in ascending raster order.
pub fn voxj_codec_main_from_vox_state(state: &VoxState) -> VoxjCodecMain {
    let objects = state
        .iter_objects()
        .map(|(_, object)| voxj_codec_object_from_vox_object(object))
        .collect();

    let palettes = state
        .iter_palettes()
        .map(|(_, palette)| voxj_palette_from_vox_palette(palette))
        .collect();

    let hierarchy_nodes = state
        .iter_hierarchy_nodes()
        .map(|(_, node)| voxj_hierarchy_node_from_vox_hierarchy_node(node))
        .collect();

    let root_hierarchy_nodes = state
        .root_hierarchy_nodes()
        .iter()
        .map(|id| id.to_u32() as usize)
        .collect();

    let ext = state.ext().map(voxj_value_from_vox_value);

    VoxjCodecMain {
        objects,
        palettes,
        hierarchy_nodes,
        root_hierarchy_nodes,
        ext,
    }
}
