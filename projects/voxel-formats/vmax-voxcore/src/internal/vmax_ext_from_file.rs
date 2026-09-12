use crate::{VMaxExt, VMaxExtNode, VMaxExtObjectState, VMaxExtPalette};
use vmax::{VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxObject};

/// The ext of a read of `serde`. `palettes` holds each folded palette's
/// provenance in palette listing order. `object_data` holds each object's
/// contents filename in object listing order.
pub fn vmax_ext_from_file(
    serde: &VMaxFile,
    palettes: Vec<VMaxExtPalette>,
    object_data: Vec<Option<String>>,
) -> VMaxExt {
    let scene = &serde.scene_json_file;
    let mut scene_block = scene.clone();
    scene_block.groups = Vec::new();
    scene_block.objects = Vec::new();

    // Aligned with the hierarchy nodes: groups first, then objects.
    let mut hierarchy_nodes: Vec<VMaxExtNode> = scene.groups.iter().map(node_from_group).collect();
    hierarchy_nodes.extend(scene.objects.iter().map(node_from_object));

    let object_states = object_data
        .iter()
        .map(|data| {
            data.as_deref()
                .and_then(|data| serde.contents_files.get(data))
                .map(object_state_from_contents)
        })
        .collect();

    VMaxExt {
        scene: scene_block,
        hierarchy_nodes,
        palettes: palettes.into_iter().map(Some).collect(),
        object_states,
    }
}

/// Captures the editor state of a contents file for the ext. The tool partition
/// (`tools.vp`) is dropped: it is the object's build volume, held natively as
/// the object's grid, so it is rebuilt on write rather than stored here.
fn object_state_from_contents(data: &VMaxContentsVmaxbFile) -> VMaxExtObjectState {
    VMaxExtObjectState {
        uuid: data.uuid.clone(),
        v: data.v,
        tools: data.tools.clone().map(|mut tools| {
            tools.vp = None;
            tools
        }),
        brush: data.brush.clone(),
        cam: data.cam.clone(),
    }
}

/// The per-node provenance for a scene object. The content box is not kept; it
/// is derived on write from the object's native tight bounds.
fn node_from_object(object: &VMaxObject) -> VMaxExtNode {
    VMaxExtNode {
        id: object.id.clone(),
        parent_id: object.parent_id.clone(),
        index: Some(object.ind),
        rotation: Some(object.rotation),
        alignment: Some(object.t_al.clone()),
        pivot_face: Some(object.t_pf.clone()),
        pivot_align: Some(object.t_pa.clone()),
        selected: object.s,
    }
}

/// The per-node provenance for a scene group. The content box is not kept; it
/// is derived on write from the bounding box of the group's subtree.
fn node_from_group(group: &VMaxGroup) -> VMaxExtNode {
    VMaxExtNode {
        id: group.id.clone(),
        parent_id: group.parent_id.clone(),
        index: Some(group.ind),
        rotation: Some(group.rotation),
        alignment: Some(group.t_al.clone()),
        pivot_face: Some(group.t_pf.clone()),
        pivot_align: Some(group.t_pa.clone()),
        selected: group.s,
    }
}
