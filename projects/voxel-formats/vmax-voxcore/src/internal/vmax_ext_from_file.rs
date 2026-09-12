use crate::{VMaxExt, VMaxExtNode, VMaxExtObjectState, VMaxExtPalette, synthesized_object_state};
use branded_id::U32Id;
use std::collections::BTreeMap;
use vmax::{VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxObject};
use voxcore::{BVoxHierarchyNode, BVoxObject, BVoxPalette, VoxMain};

/// The ext of a read of `serde` into `main`.
///
/// 1. `node_ids`: the hierarchy node ids in scene order, groups then objects
/// 2. `palettes`: each folded palette's provenance by palette id
/// 3. `object_data`: each object's contents filename by object id, or `None`
///    for an object with no contents file, which takes a synthesized editor
///    state
pub fn vmax_ext_from_file(
    serde: &VMaxFile,
    main: &VoxMain<()>,
    node_ids: &[U32Id<BVoxHierarchyNode>],
    palettes: BTreeMap<U32Id<BVoxPalette>, VMaxExtPalette>,
    object_data: Vec<(U32Id<BVoxObject>, Option<String>)>,
) -> VMaxExt {
    let scene = &serde.scene_json_file;
    let mut scene_block = scene.clone();
    scene_block.groups = Vec::new();
    scene_block.objects = Vec::new();

    let entries = scene
        .groups
        .iter()
        .map(node_from_group)
        .chain(scene.objects.iter().map(node_from_object));
    let hierarchy_nodes = node_ids.iter().copied().zip(entries).collect();

    let mut ext = VMaxExt {
        scene: scene_block,
        hierarchy_nodes,
        palettes,
        object_states: BTreeMap::new(),
    };

    for (object_id, data) in object_data {
        let entry = match data.and_then(|data| serde.contents_files.get(&data)) {
            Some(contents) => object_state_from_contents(contents),
            None => {
                let object = main.object(object_id).expect("a loaded object is live");
                synthesized_object_state(&ext, object)
            }
        };
        ext.object_states.insert(object_id, entry);
    }

    ext
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

/// The per-node provenance for a scene object. The hierarchy carries the
/// parent. The write derives the content box from the object's tight bounds.
fn node_from_object(object: &VMaxObject) -> VMaxExtNode {
    VMaxExtNode {
        id: object.id.clone(),
        index: object.ind,
        rotation: object.rotation,
        alignment: object.t_al.clone(),
        pivot_face: object.t_pf.clone(),
        pivot_align: object.t_pa.clone(),
        selected: object.s,
    }
}

/// The per-node provenance for a scene group. The hierarchy carries the
/// parent. The write derives the content box from the group's subtree.
fn node_from_group(group: &VMaxGroup) -> VMaxExtNode {
    VMaxExtNode {
        id: group.id.clone(),
        index: group.ind,
        rotation: group.rotation,
        alignment: group.t_al.clone(),
        pivot_face: group.t_pf.clone(),
        pivot_align: group.t_pa.clone(),
        selected: group.s,
    }
}
