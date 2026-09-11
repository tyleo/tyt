use crate::{
    Error, FALLBACK_CONTENT_VERSION, IDENTITY_AXIS_ANGLE, Placement, Result, SYNTH_CAMERA,
    SceneCameraSource, VMaxWriteOptions,
    ext::{VMaxExt, VMaxExtNode, VMaxExtObjectState, VMaxExtPalette},
};
use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use ty_math::TyQuaternionF64;
use vmax::VMaxSceneJsonFile;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxHierarchyNode, VoxMain};

/// Default transform-anchor tokens for a synthesized node. Voxel Max decodes
/// each as an enum and rejects an empty token.
const DEFAULT_ALIGNMENT: &str = "f";

const DEFAULT_PIVOT_ALIGN: &str = "4";

const DEFAULT_PIVOT_FACE: &str = "8";

/// What the writer draws from the ext. [`VMaxExt`] supplies the loaded
/// document's. `()` synthesizes from the scene.
pub trait VMaxExtSource: VoxExt + Sized {
    /// The scene nodes to emit, each with the provenance that places it.
    /// Errors when the ext is out of step with the listings.
    fn placements(state: &VoxMain<Self>) -> Result<Vec<Placement<'_>>>;

    /// The scene-level state the document opens with, its objects and
    /// groups left empty. Errors when `options` asks for the ext's scene
    /// camera and there is no ext.
    fn scene(state: &VoxMain<Self>, options: &VMaxWriteOptions) -> Result<VMaxSceneJsonFile>;

    /// The provenance of the palette at `index`.
    fn palette(state: &VoxMain<Self>, index: usize) -> Option<&VMaxExtPalette>;

    /// The editor state of the object at `index`.
    fn object_state(state: &VoxMain<Self>, index: usize) -> Option<&VMaxExtObjectState>;
}

impl VMaxExtSource for VMaxExt {
    fn placements(state: &VoxMain<Self>) -> Result<Vec<Placement<'_>>> {
        let vmax_ext = state.ext();
        check_alignment(state, vmax_ext)?;

        Ok(ext_placements(state, vmax_ext))
    }

    fn scene(state: &VoxMain<Self>, _options: &VMaxWriteOptions) -> Result<VMaxSceneJsonFile> {
        Ok(state.ext().scene.clone())
    }

    fn palette(state: &VoxMain<Self>, index: usize) -> Option<&VMaxExtPalette> {
        state.ext().palettes[index].as_ref()
    }

    fn object_state(state: &VoxMain<Self>, index: usize) -> Option<&VMaxExtObjectState> {
        state.ext().object_states[index].as_ref()
    }
}

/// Checks that each aligned list of the ext is as long as its listing. The
/// hooks keep them in step, so a mismatch is a malformed ext.
fn check_alignment<T>(state: &VoxMain<T>, vmax_ext: &VMaxExt) -> Result<()> {
    let lists = [
        (
            "hierarchy nodes",
            vmax_ext.hierarchy_nodes.len(),
            state.hierarchy_node_count(),
        ),
        ("palettes", vmax_ext.palettes.len(), state.palette_count()),
        (
            "object states",
            vmax_ext.object_states.len(),
            state.object_count(),
        ),
    ];
    for (list, stored, count) in lists {
        if stored != count {
            return Err(Error::invalid(format!(
                "vmax ext has {stored} {list} but the state has {count}"
            )));
        }
    }
    Ok(())
}

/// Pairs each voxcore node with its ext entry by listing index, the placement
/// the lossless path emits. A vmax-origin scene is a tree with one ext node
/// per voxcore node, so this reproduces it exactly. An entry with no id was
/// inserted by a hook for a node retained after the load, so it is filled in
/// like a synthesized node: a fresh UUID no other entry uses, its first
/// parent's id, the node's rotation, and the default anchor tokens.
fn ext_placements<'a, T>(state: &'a VoxMain<T>, vmax_ext: &VMaxExt) -> Vec<Placement<'a>> {
    // Every id first, so a child can link to an inserted parent anywhere in
    // the listing.
    let taken: HashSet<&str> = vmax_ext
        .hierarchy_nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    let mut counter = 0usize;
    let ids: Vec<String> = vmax_ext
        .hierarchy_nodes
        .iter()
        .map(|node| {
            if !node.id.is_empty() {
                return node.id.clone();
            }
            loop {
                let id = synth_uuid(counter);
                counter += 1;
                if !taken.contains(id.as_str()) {
                    return id;
                }
            }
        })
        .collect();

    let mut parent_indices: HashMap<U32Id<BVoxHierarchyNode>, usize> = HashMap::new();
    for (index, (_, node)) in state.iter_hierarchy_nodes().enumerate() {
        for &child_id in &node.child_node_ids {
            parent_indices.entry(child_id).or_insert(index);
        }
    }

    state
        .iter_hierarchy_nodes()
        .enumerate()
        .map(|(index, (node_id, node))| {
            let stored = &vmax_ext.hierarchy_nodes[index];
            let ext = if stored.id.is_empty() {
                let parent_id = parent_indices
                    .get(&node_id)
                    .map(|&parent_index| ids[parent_index].clone());
                synthesized_node(ids[index].clone(), parent_id, node)
            } else {
                stored.clone()
            };
            Placement { node_id, node, ext }
        })
        .collect()
}

impl VMaxExtSource for () {
    fn placements(state: &VoxMain<Self>) -> Result<Vec<Placement<'_>>> {
        Ok(synthesize_placements(state))
    }

    /// A fallback scene version and a neutral camera. Errors on the `ext` scene
    /// camera because a bare state carries none.
    fn scene(_state: &VoxMain<Self>, options: &VMaxWriteOptions) -> Result<VMaxSceneJsonFile> {
        if matches!(options.scene_camera, Some(SceneCameraSource::Ext)) {
            return Err(Error::invalid(
                "scene camera `ext` needs a vmax ext, which the input has none of",
            ));
        }

        Ok(VMaxSceneJsonFile {
            v: FALLBACK_CONTENT_VERSION,
            cam: Some(SYNTH_CAMERA),
            ..Default::default()
        })
    }

    fn palette(_state: &VoxMain<Self>, _index: usize) -> Option<&VMaxExtPalette> {
        None
    }

    fn object_state(_state: &VoxMain<Self>, _index: usize) -> Option<&VMaxExtObjectState> {
        None
    }
}

/// Walks the hierarchy from the roots, emitting one [`Placement`] per node-path
/// with a synthesized ext, so the reverse path can rebuild a Voxel Max document
/// from a state that carries no `vmax` ext. A subtree shared by several
/// parents is duplicated per path, matching the way voxcore composes a node's
/// placement along every path to it, so the rebuilt world is identical even
/// though Voxel Max models only a tree. Instances collapse back to one shared
/// object when the reverse path dedups them.
///
/// Lossy only where Voxel Max cannot represent the data from a bare scene: node
/// rotation is dropped to identity, since the reverse path stores an axis-angle
/// the voxcore quaternion is not inverted to, and the material palette name is
/// left empty. Node translation and scale, the hierarchy, colors, and any
/// material palette survive.
fn synthesize_placements<T>(state: &VoxMain<T>) -> Vec<Placement<'_>> {
    let mut placements = Vec::new();
    let mut counter = 0usize;
    for &root_id in state.root_hierarchy_node_ids() {
        push_placement(state, root_id, None, &mut counter, &mut placements);
    }
    placements
}

/// Emits a placement for the node `node_id` under `parent_id`, then recurses
/// into its child nodes. Each occurrence takes a fresh synthesized UUID, so a
/// node reached by several paths becomes a distinct scene node per path; child
/// nodes attach to this occurrence's id, which is also the id of the node's
/// first object.
fn push_placement<'a, T>(
    state: &'a VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent_id: Option<String>,
    counter: &mut usize,
    placements: &mut Vec<Placement<'a>>,
) {
    let node = state
        .hierarchy_node(node_id)
        .expect("a valid hierarchy node");
    let ext_id = synth_uuid(*counter);
    *counter += 1;
    let ext = synthesized_node(ext_id.clone(), parent_id, node);
    placements.push(Placement { node_id, node, ext });
    for &child_id in &node.child_node_ids {
        push_placement(state, child_id, Some(ext_id.clone()), counter, placements);
    }
}

/// The provenance of a node the document never carried. The content box and
/// placement are derived from the native bounds and node transform on write,
/// so it holds the ids, the node's rotation encoded from its live quaternion,
/// and the default anchor tokens.
fn synthesized_node(id: String, parent_id: Option<String>, node: &VoxHierarchyNode) -> VMaxExtNode {
    VMaxExtNode {
        id,
        parent_id,
        index: None,
        rotation: Some(axis_angle(node.transform.rotation)),
        alignment: Some(DEFAULT_ALIGNMENT.to_owned()),
        pivot_face: Some(DEFAULT_PIVOT_FACE.to_owned()),
        pivot_align: Some(DEFAULT_PIVOT_ALIGN.to_owned()),
        selected: None,
    }
}

/// The `[x, y, z, angle]` axis-angle that reproduces a quaternion rotation, the
/// inverse of [`ext_rotation`]. A synthesized node has no preserved `t_r`, so
/// its rotation is encoded from the live quaternion; feeding the result back
/// through [`ext_rotation`] (and Voxel Max's own decode) recovers the same
/// rotation.
fn axis_angle(rotation: TyQuaternionF64) -> [f64; 4] {
    let (axis, angle) = rotation.to_axis_angle();
    if angle == 0.0 {
        // No rotation: match Voxel Max's `[0, 0, 0, 0]` rather than emit a bare
        // axis.
        return IDENTITY_AXIS_ANGLE;
    }
    [axis.x, axis.y, axis.z, angle]
}

/// A syntactically valid, deterministic UUID for a synthesized scene node.
/// Voxel Max decodes a node's `id`/`pid` as a UUID and rejects a non-UUID
/// token. The index is offset by one so the first node avoids the all-zero nil
/// UUID.
fn synth_uuid(index: usize) -> String {
    format!("00000000-0000-0000-0000-{:012X}", index + 1)
}
