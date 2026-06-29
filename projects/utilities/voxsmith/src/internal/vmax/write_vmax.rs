use crate::{
    Error, Result, SceneCameraSource, VoxelMaxColorFormat, VoxelMaxExt, VoxelMaxExtWrapper,
    VoxelMaxNode, VoxelMaxPalette, ext_for, from_vox_value, tighten,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, HashMap};
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use vmax::{
    VMaxBrush, VMaxBrushColor, VMaxBrushEntry, VMaxBrushState, VMaxCamera, VMaxContentsVmaxbFile,
    VMaxFile, VMaxFlag, VMaxFlagValue, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxMode,
    VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
    VMaxSceneJsonFile, VMaxToolMode, VMaxTools, VMaxViewBox,
};
use vmax_codec::{VMaxVoxel, encode_vmax_snapshots};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteRef, VoxHierarchyNode, VoxMain,
    VoxObject, VoxPalette, VoxValue,
};

/// Usable colors in a Voxel Max palette. Color indices are 1-based: `color_idx`
/// is `cell + 1`, runs 1..=255, and 0 is the empty cell. Colors are stored
/// 0-based; a `palette*.png` appends a transparent terminator (256 entries),
/// the plist `colors` table does not (255 entries).
const PALETTE_COLORS: usize = 255;

/// Codable version stamped on a rebuilt contents file when the state carries no
/// preserved object version.
const FALLBACK_CONTENT_VERSION: i64 = 4;

/// Axis-angle stored on a node with no preserved rotation; a degenerate axis
/// decodes to the identity quaternion.
const IDENTITY_AXIS_ANGLE: [f64; 4] = [0.0, 0.0, 0.0, 0.0];

/// Default transform-anchor tokens for a synthesized node. Voxel Max decodes
/// each as an enum and rejects an empty token.
const DEFAULT_ALIGNMENT: &str = "f";
const DEFAULT_PIVOT_ALIGN: &str = "4";
const DEFAULT_PIVOT_FACE: &str = "8";

/// The `pal` an object with no color palette borrows. An empty reference makes
/// Voxel Max read the package directory as a file and abort, so a colorless
/// object shares the first color palette's name and writes no file of its own.
const FALLBACK_PALETTE: &str = "palette.png";

/// The scene camera a synthesized document opens with, mirroring a fresh Voxel
/// Max document's neutral rig. Voxel Max needs a valid rig to present an
/// imported document; the framing is cosmetic.
const SYNTH_CAMERA: VMaxSceneCamera = VMaxSceneCamera {
    da: 0.0,
    ha: 0.25,
    lda: 0.0,
    lha: 1.875,
    lwa: 0.25,
    o: [0.0, 0.0, 0.0],
    px: 0.0,
    py: 0.0,
    wa: 0.0,
    z: 512.0,
};

/// Writes a [`VoxMain`] back to a Voxel Max document, the workhorse behind
/// [`to_vmax_file`](crate::to_vmax_file) and
/// [`VmaxFileBuilder`](crate::VmaxFileBuilder).
///
/// A state carrying the `voxel-max` ext the forward path writes is rebuilt from
/// it, and editor session artifacts voxcore does not model are dropped. A state
/// without that ext, such as one loaded from another format, has its ext
/// synthesized from the bare voxcore scene by [`synthesize_voxel_max_ext`] and
/// the rest of this path runs unchanged. `scene_camera` overrides the scene
/// camera the document opens with, or keeps the path's own when `None`.
pub fn write_vmax(
    state: &VoxMain,
    voxel_max_color_format: VoxelMaxColorFormat,
    scene_camera: Option<SceneCameraSource>,
) -> Result<VMaxFile> {
    let had_ext = ext_for(state, "voxel-max").is_some();
    let (voxel_max, placements) = match ext_for(state, "voxel-max") {
        Some(ext) => {
            let voxel_max = from_vox_value::<VoxelMaxExtWrapper>(ext)?.voxel_max;
            let placements = ext_placements(state, &voxel_max);
            (voxel_max, placements)
        }
        None => {
            let voxel_max = synthesize_voxel_max_ext(state);
            let placements = synthesize_placements(state);
            (voxel_max, placements)
        }
    };

    let mut objects: Vec<VMaxObject> = Vec::new();
    let mut groups: Vec<VMaxGroup> = Vec::new();
    // Color palette id -> the `pal` filename written for it.
    let mut palette_files: HashMap<usize, String> = HashMap::new();
    // Object id -> its `data` filename, shared by every node that places it.
    let mut contents_by_object: HashMap<usize, String> = HashMap::new();

    let mut contents_files: BTreeMap<String, VMaxContentsVmaxbFile> = BTreeMap::new();
    let mut palette_settings_files: BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile> =
        BTreeMap::new();
    let mut palette_png_files: BTreeMap<String, VMaxPalettePngFile> = BTreeMap::new();

    // A distinct `ind` per emitted node; Voxel Max collapses nodes that share
    // `[0, 0, 0]`. The lossless path keeps its preserved `ind` and skips this.
    let mut ind_counter = 0i64;

    // A group's content box is derived from its subtree, the same box for every
    // path to a shared node, so it is memoized by node id.
    let mut box_memo: HashMap<u32, ([f64; 3], [f64; 3])> = HashMap::new();

    for placement in &placements {
        let node = placement.node;
        let ext_node = &placement.ext;

        if node.child_objects.is_empty() {
            let ind = node_ind(ext_node, true, &mut ind_counter);
            let (center, half) = subtree_box_local(state, placement.id, &mut box_memo);
            groups.push(group_from_node(node, ext_node, ind, center, half));
            continue;
        }

        // One scene object per child object. A vmax-origin node always carries
        // a single object, so this loops once and matches the lossless path
        // exactly. A synthesized node may carry several, such as a Goxel
        // layer's blocks; the extra objects become sibling object-nodes under
        // the same parent.
        for (slot, object_ref) in node.child_objects.iter().enumerate() {
            let object_ref = *object_ref;
            let object_id = object_ref.to_u32() as usize;
            let object = state.object(object_ref).expect("a valid node child object");
            let (color, material) = object_palette_refs(state, object);
            let suffix = suffix(object_id);
            // The node's ext places its first object; an extra object gets a
            // per-object variant with a distinct id and its own bounds.
            let object_ext = if slot == 0 {
                ext_node.clone()
            } else {
                secondary_object_ext(ext_node, slot)
            };

            // Re-derive the object's internal-grid placement by convention:
            // center the canvas in the 256-wide workspace, then seat the
            // runtime grid inside it by the runtime/edit origin offset. The
            // runtime grid is the live voxels' tight extent within the object's
            // build volume; the content box follows from it and the build
            // volume, the scene placement from the node transform.
            let (tight, (edit_bounds, edit_origin)) = tighten(object);
            let placement =
                object_placement(tight.bounds(), tight.origin(), edit_bounds, edit_origin);
            let object_state = voxel_max
                .object_states
                .get(object_id)
                .and_then(|s| s.clone());

            // Instances share one contents file: rebuild it once.
            let data = match contents_by_object.get(&object_id) {
                Some(data) => data.clone(),
                None => {
                    let voxels = reconstruct_voxels(&tight, color, material, placement.box_min)?;
                    let data = format!("contents{suffix}.vmaxb");
                    // Voxels re-encode into snapshots; serde-only state the
                    // decoded voxcore object does not model (`pal`) stays
                    // absent.
                    let contents = match object_state {
                        // Keep the preserved editor session, but re-scope its
                        // canvas (`vp`) to the derived build volume.
                        Some(state) => {
                            let mut tools = state.tools;
                            if let Some(tools) = tools.as_mut() {
                                tools.vp = Some(placement.view_box.clone());
                            }
                            VMaxContentsVmaxbFile {
                                snapshots: encode_vmax_snapshots(&voxels),
                                uuid: state.uuid,
                                v: state.v,
                                tools,
                                brush: state.brush,
                                cam: state.cam,
                                pal: None,
                            }
                        }
                        // Voxel Max's object decoder rejects a sparse editor
                        // state, so a synthesized object carries default
                        // `tools`, `brush`, and `cam`. Its `vp` frames the
                        // object's build volume.
                        None => VMaxContentsVmaxbFile {
                            snapshots: encode_vmax_snapshots(&voxels),
                            uuid: object_ext.id.clone(),
                            v: FALLBACK_CONTENT_VERSION,
                            tools: Some(default_tools(placement.view_box.clone())),
                            brush: Some(default_brush()),
                            cam: Some(default_camera(placement.center)),
                            pal: None,
                        },
                    };
                    contents_files.insert(data.clone(), contents);
                    contents_by_object.insert(object_id, data.clone());
                    data
                }
            };

            let pal = build_palette(
                state,
                &voxel_max.palettes,
                color.map(|(_, palette)| palette),
                material.map(|(_, palette)| palette),
                &mut palette_files,
                &mut palette_settings_files,
                &mut palette_png_files,
                voxel_max_color_format,
            );

            let ind = node_ind(&object_ext, false, &mut ind_counter);
            objects.push(object_from_node(
                node,
                &object_ext,
                &placement,
                data,
                pal,
                &suffix,
                ind,
            ));
        }
    }

    let mut scene = voxel_max.scene;
    scene.groups = groups;
    scene.objects = objects;
    apply_scene_camera(&mut scene, scene_camera, had_ext)?;

    Ok(VMaxFile {
        scene_json_file: scene,
        contents_files,
        palette_settings_files,
        palette_png_files,
        history_vmaxhb_files: BTreeMap::new(),
        history_vmaxhvsb_files: BTreeMap::new(),
        history_vmaxhvsc_files: BTreeMap::new(),
        selection_vmaxb_files: BTreeMap::new(),
        thumbnail_png: None,
        contents_vmax_pngs: BTreeMap::new(),
        group_pngs: BTreeMap::new(),
    })
}

/// Default `tools` for a synthesized object. Voxel Max's object decoder rejects
/// a sparse `tools`, so this mirrors a fresh object's editor state. `view_box`
/// scopes the view/edit partition (`vp`) to the object's build volume.
fn default_tools(view_box: VMaxViewBox) -> VMaxTools {
    // A mode dict that sets only `mo`, or only `m`.
    let mo = |mo: &str| VMaxMode {
        mo: Some(mo.to_owned()),
        ..Default::default()
    };
    let m = |m: &str| VMaxMode {
        m: Some(m.to_owned()),
        ..Default::default()
    };
    let flag = |x: VMaxFlagValue| {
        Some(VMaxFlag {
            x,
            y: None,
            z: None,
        })
    };
    // A tool with a single active surface; the closure picks the surface field.
    fn tool(set: impl FnOnce(&mut VMaxToolMode)) -> Option<VMaxToolMode> {
        let mut mode = VMaxToolMode::default();
        set(&mut mode);
        Some(mode)
    }
    VMaxTools {
        bs: 1,
        mi: 0,
        bi: 0,
        al: "1".to_owned(),
        src: None,
        stf: flag(VMaxFlagValue::Int(1)),
        mr: flag(VMaxFlagValue::Bool(false)),
        st: flag(VMaxFlagValue::Bool(false)),
        vp: Some(view_box),
        bst: Some(VMaxBrushState {
            cm: "ng".to_owned(),
            cp: "n".to_owned(),
            gm: "u".to_owned(),
            gp: "n".to_owned(),
            ocx: Some(0),
            ocn: Some(-1),
            sfaz: None,
            sfat: None,
        }),
        ct: tool(|t| t.c = Some(mo("v"))),
        ctc: tool(|t| t.c = Some(mo("v"))),
        cte: tool(|t| t.e = Some(mo("v"))),
        ctp: tool(|t| t.p = Some(mo("v"))),
        cts: tool(|t| {
            t.s = Some(VMaxMode {
                mo: Some("v".to_owned()),
                mf: Some("nw".to_owned()),
                ..Default::default()
            })
        }),
        ctm: tool(|t| t.m = Some(mo("d"))),
        pctm: tool(|t| t.m = Some(mo("d"))),
        cta: tool(|t| t.a = Some(mo("ma"))),
        dm: tool(|t| t.b = Some(m("d"))),
        dmb: tool(|t| t.b = Some(m("d"))),
        dmc: tool(|t| t.c = Some(m("e"))),
        dml: tool(|t| t.l = Some(m("d"))),
        dms: tool(|t| {
            t.s = Some(VMaxMode {
                m: Some("c8".to_owned()),
                t: Some("f".to_owned()),
                ..Default::default()
            })
        }),
    }
}

/// Default `brush` palette for a synthesized object, mirroring a fresh
/// document.
fn default_brush() -> VMaxBrush {
    use VMaxBrushEntry::{Bb, C, Ch, Db, E, Eh, Pr, Py};
    let color = |dm: [i64; 3]| VMaxBrushColor { dm: dm.to_vec() };
    VMaxBrush {
        name: "Palette #1".to_owned(),
        current: 0,
        brushes: vec![
            C(color([1, 1, 1])),
            Ch(color([5, 5, 5])),
            E(color([5, 5, 5])),
            Eh(color([5, 5, 5])),
            Bb(color([5, 5, 5])),
            Db(color([5, 5, 5])),
            Pr(color([5, 5, 5])),
            Py(color([5, 5, 5])),
        ],
    }
}

/// Default editor `cam` for a synthesized object. A single-object document
/// opens straight into this object-editor view, so the camera must orbit the
/// object's content, not the corner of its 0..256 internal grid: `target` is
/// the content center in internal-grid coordinates (the object's `e_c`),
/// matching the rig Voxel Max writes when it frames the object. The rest is a
/// neutral framed-view rig.
fn default_camera(target: [f64; 3]) -> VMaxCamera {
    VMaxCamera {
        wa: 0.0,
        ha: 0.1959133446216583,
        da: 0.0,
        lwa: 0.25,
        lha: 1.820913314819336,
        lda: 0.0,
        px: 0.0,
        py: 0.0,
        z: 512.0,
        o: target,
    }
}

/// Applies a scene-camera override to the rebuilt scene. The lossless path
/// carries the ext's camera and the synthesis path the empty default, so `None`
/// leaves the scene untouched.
fn apply_scene_camera(
    scene: &mut VMaxSceneJsonFile,
    scene_camera: Option<SceneCameraSource>,
    had_ext: bool,
) -> Result<()> {
    match scene_camera {
        None => {}
        Some(SceneCameraSource::Ext) if !had_ext => {
            return Err(Error::invalid(
                "scene camera `ext` needs a voxel-max ext, which the input has none of",
            ));
        }
        Some(SceneCameraSource::Ext) => {}
        Some(SceneCameraSource::Empty) => scene.cam = Some(SYNTH_CAMERA),
        Some(SceneCameraSource::Camera(camera)) => scene.cam = Some(camera),
    }
    Ok(())
}

/// One scene node to emit and the Voxel Max provenance that places it: the
/// voxcore node supplies the local transform, the ext supplies the id, parent,
/// and bounds. The lossless path pairs each voxcore node with its ext entry by
/// index; synthesis walks the hierarchy from the roots, so a node shared by
/// several parents, or one that is both a root and a child, is emitted once per
/// path the way voxcore renders it, and a node reachable from no root is
/// dropped just as voxcore never places it.
struct Placement<'a> {
    id: U32Id<BVoxHierarchyNode>,
    node: &'a VoxHierarchyNode,
    ext: VoxelMaxNode,
}

/// Pairs each voxcore node with its ext entry by index, the placement the
/// lossless path emits. A vmax-origin scene is a tree with one ext node per
/// voxcore node, so this reproduces it exactly.
fn ext_placements<'a>(state: &'a VoxMain, voxel_max: &VoxelMaxExt) -> Vec<Placement<'a>> {
    state
        .iter_hierarchy_nodes()
        .enumerate()
        .map(|(index, (id, node))| Placement {
            id,
            node,
            ext: voxel_max
                .hierarchy_nodes
                .get(index)
                .cloned()
                .unwrap_or_default(),
        })
        .collect()
}

/// Walks the hierarchy from the roots, emitting one [`Placement`] per node-path
/// with a synthesized ext, so the reverse path can rebuild a Voxel Max document
/// from a state that carries no `voxel-max` ext. A subtree shared by several
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
fn synthesize_placements(state: &VoxMain) -> Vec<Placement<'_>> {
    let mut placements = Vec::new();
    let mut counter = 0usize;
    for &root in state.root_hierarchy_nodes() {
        push_placement(state, root, None, &mut counter, &mut placements);
    }
    placements
}

/// Emits a placement for the node `id` under `parent_id`, then recurses into
/// its child nodes. Each occurrence takes a fresh synthesized UUID, so a node
/// reached by several paths becomes a distinct scene node per path; child nodes
/// attach to this occurrence's id, which is also the id of the node's first
/// object.
fn push_placement<'a>(
    state: &'a VoxMain,
    id: U32Id<BVoxHierarchyNode>,
    parent_id: Option<String>,
    counter: &mut usize,
    placements: &mut Vec<Placement<'a>>,
) {
    let node = state.hierarchy_node(id).expect("a valid hierarchy node");
    let ext_id = synth_uuid(*counter);
    *counter += 1;
    // The content box and placement are derived from the native bounds and node
    // transform on write, so a synthesized node carries the default anchor
    // tokens; its rotation is encoded from the node's live quaternion, and its
    // parent links this occurrence's id.
    let ext = VoxelMaxNode {
        id: ext_id.clone(),
        parent_id,
        index: None,
        rotation: Some(axis_angle(node.transform.rotation)),
        alignment: Some(DEFAULT_ALIGNMENT.to_owned()),
        pivot_face: Some(DEFAULT_PIVOT_FACE.to_owned()),
        pivot_align: Some(DEFAULT_PIVOT_ALIGN.to_owned()),
        selected: None,
    };
    placements.push(Placement { id, node, ext });
    for &child in &node.child_nodes {
        push_placement(state, child, Some(ext_id.clone()), counter, placements);
    }
}

/// Synthesizes the scene-level `voxel-max` ext for a state that carries none,
/// such as one loaded from another format. It holds only the document-wide data
/// with no per-node home: a fallback scene version, a neutral camera, an empty
/// name for each palette, and no preserved object state. Per-node provenance
/// comes from [`synthesize_placements`].
fn synthesize_voxel_max_ext(state: &VoxMain) -> VoxelMaxExt {
    VoxelMaxExt {
        scene: VMaxSceneJsonFile {
            v: FALLBACK_CONTENT_VERSION,
            cam: Some(SYNTH_CAMERA),
            ..Default::default()
        },
        hierarchy_nodes: Vec::new(),
        palettes: vec![None; state.palette_count()],
        object_states: Vec::new(),
    }
}

/// The per-object ext for an extra object on a node placing several, such as a
/// Goxel layer's blocks. It takes a distinct id and inherits the node's parent,
/// rotation, and alignment to stay a sibling of the node's first object; its
/// content box is derived from its own bounds on write.
fn secondary_object_ext(node_ext: &VoxelMaxNode, slot: usize) -> VoxelMaxNode {
    VoxelMaxNode {
        id: secondary_uuid(&node_ext.id, slot),
        ..node_ext.clone()
    }
}

/// The build volume's edit-space origin for a synthesized object grid of
/// `size`: centered in the 256-wide workspace in x and y and on the floor in z,
/// where Voxel Max frames a fresh object. A model wider than the workspace
/// keeps the origin corner so its voxels stay non-negative.
fn centered_origin(size: TyVector3U32) -> [i32; 3] {
    let centered = |s: u32| ((256 - s as i64) / 2).max(0) as i32;
    [centered(size.x), centered(size.y), 0]
}

/// An object's internal-grid placement derived for the Voxel Max write: where
/// its voxels sit in the private workspace, and the content box and build
/// volume that follow. The scene placement is recovered separately from the
/// node transform.
struct ObjectPlacement {
    /// The runtime grid's min corner in the workspace.
    box_min: [i32; 3],

    /// The grid `origin` carried over from the object, for the pivot math.
    origin: [i32; 3],

    /// The content center (`e_c`): `box_min + bounds / 2`.
    center: [f64; 3],

    /// The content box min relative to the center (`e_mi`): `-bounds / 2`.
    bounds_min: [f64; 3],

    /// The content box max relative to the center (`e_ma`): `bounds / 2`.
    bounds_max: [f64; 3],

    /// The build volume (`tools.vp`): the edit grid placed in the workspace.
    view_box: VMaxViewBox,
}

/// Derives an object's internal-grid placement by convention. The canvas (edit
/// grid) is centered in the 256-wide workspace, and the runtime grid sits
/// inside it by the runtime/edit origin offset. The content box follows from
/// the tight bounds. The internal-grid position is invisible to the scene,
/// which the node transform places, so re-deriving it loses nothing.
fn object_placement(
    bounds: TyVector3U32,
    origin: TyVector3I32,
    edit_bounds: TyVector3U32,
    edit_origin: TyVector3I32,
) -> ObjectPlacement {
    let canvas_min = centered_origin(edit_bounds);
    let origin = [origin.x, origin.y, origin.z];
    let box_min = [
        canvas_min[0] + origin[0] - edit_origin.x,
        canvas_min[1] + origin[1] - edit_origin.y,
        canvas_min[2] + origin[2] - edit_origin.z,
    ];
    // An empty runtime grid has no content box of its own; frame it on the
    // build volume so Voxel Max keeps a sensible content center and box for the
    // object.
    let (content_min, content_size) = if bounds.x == 0 && bounds.y == 0 && bounds.z == 0 {
        (canvas_min, edit_bounds)
    } else {
        (box_min, bounds)
    };
    let (center, bounds_min, bounds_max) = content_box(content_min, content_size);
    ObjectPlacement {
        box_min,
        origin,
        center,
        bounds_min,
        bounds_max,
        view_box: object_view_box(canvas_min, edit_bounds),
    }
}

/// The Voxel Max content bounds `(e_c, e_mi, e_ma)` for a grid of `bounds`
/// whose min corner sits at `box_min`: the box center and its symmetric
/// half-extents. Voxel Max renders and frames against this and pivots about the
/// center.
fn content_box(box_min: [i32; 3], bounds: TyVector3U32) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let half = [
        bounds.x as f64 / 2.0,
        bounds.y as f64 / 2.0,
        bounds.z as f64 / 2.0,
    ];
    let center = [
        box_min[0] as f64 + half[0],
        box_min[1] as f64 + half[1],
        box_min[2] as f64 + half[2],
    ];
    (center, [-half[0], -half[1], -half[2]], half)
}

/// The `tools.vp` partition box for an object: its build volume at `origin`,
/// inclusive, so Voxel Max frames the whole authored grid.
fn object_view_box(origin: [i32; 3], size: TyVector3U32) -> VMaxViewBox {
    VMaxViewBox {
        min: [origin[0] as i64, origin[1] as i64, origin[2] as i64],
        max: [
            origin[0] as i64 + size.x.max(1) as i64 - 1,
            origin[1] as i64 + size.y.max(1) as i64 - 1,
            origin[2] as i64 + size.z.max(1) as i64 - 1,
        ],
    }
}

/// A reference into one of an object's palettes: the sample reference id and
/// the palette id it names.
type PaletteRef = (U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>);

/// The color and material palette references for an object, identified by the
/// `rgba` and `metallic` attributes.
fn object_palette_refs(
    state: &VoxMain,
    object: &VoxObject,
) -> (Option<PaletteRef>, Option<PaletteRef>) {
    let mut color = None;
    let mut material = None;
    for (reference, palette) in object.iter_palette_refs() {
        let Some(palette_value) = state.palette(palette) else {
            continue;
        };
        if has_attribute(palette_value, "rgba") || has_attribute(palette_value, "rgb") {
            color = Some((reference, palette));
        } else if has_attribute(palette_value, "metallic") {
            material = Some((reference, palette));
        }
    }
    (color, material)
}

/// Whether `palette` declares an attribute named `name`.
fn has_attribute(palette: &VoxPalette, name: &str) -> bool {
    palette
        .iter_attributes()
        .any(|(_, attribute)| attribute == name)
}

/// Re-bases an object's voxels to absolute model space, reading the color and
/// material indices from their sample references. A voxel's `color_idx` is the
/// 1-based `cell + 1`; a colorless voxel takes index 1.
///
/// Errors when a voxel's color cell reaches [`PALETTE_COLORS`], one past the
/// last usable color, so a padded source palette (such as MagicaVoxel's fixed
/// 256) is fine as long as its referenced colors fit.
fn reconstruct_voxels(
    object: &VoxObject,
    color: Option<PaletteRef>,
    material: Option<PaletteRef>,
    box_min: [i32; 3],
) -> Result<Vec<VMaxVoxel>> {
    object
        .iter_live()
        .map(|voxel| {
            let position = object
                .voxel_position(voxel)
                .expect("a live voxel is within the grid");
            let cell = |reference: Option<PaletteRef>| {
                reference.map(|(reference, _)| {
                    object
                        .voxel_cell(voxel, reference)
                        .expect("a live voxel samples every reference")
                        .to_u32()
                })
            };
            let color_idx = match cell(color) {
                Some(cell) if cell >= PALETTE_COLORS as u32 => {
                    return Err(Error::invalid(format!(
                        "a voxel references color cell {cell}, but a Voxel Max palette holds \
                         only {PALETTE_COLORS} colors, so the source has more colors than fit"
                    )));
                }
                Some(cell) => cell + 1,
                // A colorless voxel still needs a non-empty index, so it takes
                // the borrowed palette's first color (1), not the empty index
                // 0.
                None => 1,
            };
            Ok(VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx: cell(material).unwrap_or(0) as u8,
                color_idx: color_idx as u8,
            })
        })
        .collect()
}

/// Returns the `pal` filename for an object, building its color image and
/// material sidecar the first time the color palette is seen. An object with no
/// color palette borrows the default palette name and writes no file.
#[allow(clippy::too_many_arguments)]
fn build_palette(
    state: &VoxMain,
    palette_names: &[Option<VoxelMaxPalette>],
    color: Option<U32Id<BVoxPalette>>,
    material: Option<U32Id<BVoxPalette>>,
    palette_files: &mut HashMap<usize, String>,
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    voxel_max_color_format: VoxelMaxColorFormat,
) -> String {
    let Some(color) = color else {
        // An object with no color palette borrows the default palette name; an
        // empty reference is one Voxel Max cannot resolve. No file is written
        // for it.
        return FALLBACK_PALETTE.to_owned();
    };
    let color_id = color.to_u32() as usize;
    if let Some(name) = palette_files.get(&color_id) {
        return name.clone();
    }
    let stem = match palette_files.len() {
        0 => String::new(),
        n => n.to_string(),
    };
    let pal = format!("palette{stem}.png");

    let colors = color_palette_colors(state, color);
    if matches!(
        voxel_max_color_format,
        VoxelMaxColorFormat::Png | VoxelMaxColorFormat::All
    ) {
        // 256 entries: the 255 colors 0-based then a transparent terminator.
        let mut cells = colors.clone();
        cells.push([0, 0, 0, 0]);
        palette_png_files.insert(pal.clone(), VMaxPalettePngFile(cells));
    }
    // The settings sidecar carries the material, and the colors when no image
    // does. Plist mode writes no image, so even a color-only object with no
    // material writes its colors here rather than dropping them.
    let write_sidecar =
        material.is_some() || matches!(voxel_max_color_format, VoxelMaxColorFormat::Plist);
    if write_sidecar {
        let name = material
            .and_then(|material| {
                palette_names
                    .get(material.to_u32() as usize)
                    .cloned()
                    .flatten()
            })
            .map(|palette| palette.name)
            .unwrap_or_default();
        let sidecar = format!("palette{stem}.settings.vmaxpsb");
        // The plist `colors` table is the 255 colors with no terminator.
        let sidecar_colors = match voxel_max_color_format {
            VoxelMaxColorFormat::Png => Vec::new(),
            VoxelMaxColorFormat::Plist | VoxelMaxColorFormat::All => colors,
        };
        palette_settings_files.insert(
            sidecar,
            material_settings(state, material, name, sidecar_colors),
        );
    }
    palette_files.insert(color_id, pal.clone());
    pal
}

/// A color palette's cells as exactly [`PALETTE_COLORS`] 0-based RGBA entries,
/// padded with transparent cells or truncated to that count. Cells past the
/// budget are dropped; a voxel that would reference one is rejected by
/// [`reconstruct_voxels`].
fn color_palette_colors(state: &VoxMain, palette: U32Id<BVoxPalette>) -> Vec<[u8; 4]> {
    let palette = state.palette(palette).expect("a referenced palette");
    let rgba = palette
        .iter_attributes()
        .find(|(_, name)| *name == "rgba" || *name == "rgb")
        .map(|(id, _)| id);
    let mut cells: Vec<[u8; 4]> = Vec::new();
    if let Some(rgba) = rgba {
        cells.extend(
            palette
                .iter_cells()
                .take(PALETTE_COLORS)
                .map(|cell| parse_rgba(palette.cell_value(cell, rgba))),
        );
    }
    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    cells
}

/// Builds a settings sidecar carrying `colors` and, when `palette` names a
/// material palette, a material per cell read from its
/// `metallic`/`roughness`/`emissive`/`shadows` attributes and the optional
/// dispersion columns. A color-only palette passes `None` and gets no
/// materials. The editor-state keys are filled with the defaults Voxel Max
/// expects, and each slot's `mi` token from its 1-based position.
fn material_settings(
    state: &VoxMain,
    palette: Option<U32Id<BVoxPalette>>,
    name: String,
    colors: Vec<[u8; 4]>,
) -> VMaxPaletteSettingsVmaxpsbFile {
    let materials = palette
        .map(|palette| {
            let palette = state.palette(palette).expect("a referenced palette");
            let attributes: HashMap<String, _> = palette
                .iter_attributes()
                .map(|(id, attribute)| (attribute.to_owned(), id))
                .collect();
            palette
                .iter_cells()
                .enumerate()
                .map(|(slot, cell)| {
                    let value = |name: &str| {
                        attributes
                            .get(name)
                            .and_then(|&id| palette.cell_value(cell, id))
                    };
                    let number = |name: &str, default: f64| match value(name) {
                        Some(VoxValue::Number(n)) => *n,
                        _ => default,
                    };
                    let dispersed = ["ior", "transmission", "absorption"]
                        .iter()
                        .any(|name| matches!(value(name), Some(VoxValue::Number(_))));
                    VMaxMaterial {
                        mi: (slot + 1).to_string(),
                        mc: number("metallic", 0.0),
                        rc: number("roughness", 0.0),
                        sic: number("emissive", 0.0),
                        sh: matches!(value("shadows"), Some(VoxValue::Bool(true))),
                        tc: None,
                        md: dispersed.then(|| VMaxMaterialDispersion {
                            absorption: number("absorption", 0.0),
                            ior: number("ior", 1.5),
                            transmission: number("transmission", 0.0),
                        }),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    VMaxPaletteSettingsVmaxpsbFile {
        name,
        materials,
        colors: colors.iter().flatten().copied().collect(),
        indices: Vec::new(),
        lc: vec![0u8; 256],
        palette_type: 0,
        transparency: 1.0,
        r: 0,
        rt: "n".to_owned(),
        cmt: "ng".to_owned(),
        current: 0,
        ali: "1".to_owned(),
        voxmats: Vec::new(),
        ls: Vec::new(),
    }
}

/// Parses a `#RRGGBB` or `#RRGGBBAA` color cell into RGBA bytes. A missing
/// alpha, as in the 6-hex form an `rgb` source carries, defaults to opaque.
fn parse_rgba(cell: Option<&VoxValue>) -> [u8; 4] {
    let Some(VoxValue::Text(hex)) = cell else {
        return [0, 0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |i: usize| {
        hex.get(i * 2..i * 2 + 2)
            .and_then(|h| u8::from_str_radix(h, 16).ok())
    };
    [
        byte(0).unwrap_or(0),
        byte(1).unwrap_or(0),
        byte(2).unwrap_or(0),
        byte(3).unwrap_or(255),
    ]
}

/// Builds a scene object from its node and preserved provenance.
fn object_from_node(
    node: &VoxHierarchyNode,
    ext_node: &VoxelMaxNode,
    placement: &ObjectPlacement,
    data: String,
    pal: String,
    suffix: &str,
    ind: [i64; 3],
) -> VMaxObject {
    VMaxObject {
        name: node.name.clone(),
        data,
        palette: pal,
        history: format!("history{suffix}.vmaxhb"),
        id: ext_node.id.clone(),
        parent_id: ext_node.parent_id.clone(),
        hidden: None,
        position: unbake_position(&node.transform, ext_rotation(ext_node), placement),
        rotation: ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE),
        scale: vector(node.transform.scale),
        ind,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone().unwrap_or_default(),
        t_pa: ext_node.pivot_align.clone().unwrap_or_default(),
        t_pf: ext_node.pivot_face.clone().unwrap_or_default(),
        t_po: None,
        center: placement.center,
        bounds_min: Some(placement.bounds_min),
        bounds_max: Some(placement.bounds_max),
    }
}

/// The bounding box `(center, half)` of all geometry under `node_id`, in that
/// node's own local frame: the union of each child object's content box and
/// each child node's box mapped through the child's transform. Voxel Max stores
/// this per group as `e_c`/`e_mi`/`e_ma`; it is the union of the subtree, so it
/// is derived here rather than kept in the ext. Memoized by node id so a
/// subtree shared across parents is walked once. A node with no geometry
/// collapses to a zero box.
fn subtree_box_local(
    state: &VoxMain,
    node_id: U32Id<BVoxHierarchyNode>,
    memo: &mut HashMap<u32, ([f64; 3], [f64; 3])>,
) -> ([f64; 3], [f64; 3]) {
    if let Some(&box_local) = memo.get(&node_id.to_u32()) {
        return box_local;
    }
    let node = state
        .hierarchy_node(node_id)
        .expect("a valid hierarchy node");
    let mut bounds: Option<([f64; 3], [f64; 3])> = None;
    for &object in &node.child_objects {
        let (center, half) = object_box_local(state, object);
        extend_bounds(&mut bounds, center, half);
    }
    for &child in &node.child_nodes {
        let (child_center, child_half) = subtree_box_local(state, child, memo);
        let transform = state
            .hierarchy_node(child)
            .expect("a valid child node")
            .transform;
        let center = transform_point(&transform, child_center);
        let half = transform_half(&transform, child_half);
        extend_bounds(&mut bounds, center, half);
    }
    let (min, max) = bounds.unwrap_or(([0.0; 3], [0.0; 3]));
    let box_local = (
        [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ],
        [
            (max[0] - min[0]) / 2.0,
            (max[1] - min[1]) / 2.0,
            (max[2] - min[2]) / 2.0,
        ],
    );
    memo.insert(node_id.to_u32(), box_local);
    box_local
}

/// An object's content box `(center, half)` in its placing node's local voxel
/// frame: the tight runtime grid `[origin, origin + bounds]`. An empty object
/// has no runtime extent of its own, so it frames its build volume instead,
/// matching the content box the write path gives it.
fn object_box_local(state: &VoxMain, object_id: U32Id<BVoxObject>) -> ([f64; 3], [f64; 3]) {
    let object = state.object(object_id).expect("a valid child object");
    let (tight, (edit_bounds, edit_origin)) = tighten(object);
    let bounds = tight.bounds();
    if bounds.x == 0 && bounds.y == 0 && bounds.z == 0 {
        let half = [
            edit_bounds.x as f64 / 2.0,
            edit_bounds.y as f64 / 2.0,
            edit_bounds.z as f64 / 2.0,
        ];
        return (
            [
                edit_origin.x as f64 + half[0],
                edit_origin.y as f64 + half[1],
                edit_origin.z as f64 + half[2],
            ],
            half,
        );
    }
    let half = [
        bounds.x as f64 / 2.0,
        bounds.y as f64 / 2.0,
        bounds.z as f64 / 2.0,
    ];
    let origin = tight.origin();
    (
        [
            origin.x as f64 + half[0],
            origin.y as f64 + half[1],
            origin.z as f64 + half[2],
        ],
        half,
    )
}

/// Grows the running `(min, max)` AABB to include the box centered at `center`
/// with half-extents `half`.
fn extend_bounds(bounds: &mut Option<([f64; 3], [f64; 3])>, center: [f64; 3], half: [f64; 3]) {
    let lo = [
        center[0] - half[0],
        center[1] - half[1],
        center[2] - half[2],
    ];
    let hi = [
        center[0] + half[0],
        center[1] + half[1],
        center[2] + half[2],
    ];
    match bounds {
        Some((min, max)) => {
            for k in 0..3 {
                min[k] = min[k].min(lo[k]);
                max[k] = max[k].max(hi[k]);
            }
        }
        None => *bounds = Some((lo, hi)),
    }
}

/// Maps a point through a node transform: scale, then rotate, then translate.
fn transform_point(transform: &TyTransformF64, point: [f64; 3]) -> [f64; 3] {
    let scaled = TyVector3F64::new(
        point[0] * transform.scale.x,
        point[1] * transform.scale.y,
        point[2] * transform.scale.z,
    );
    let rotated = transform.rotation.rotate(scaled);
    [
        transform.position.x + rotated.x,
        transform.position.y + rotated.y,
        transform.position.z + rotated.z,
    ]
}

/// The half-extent of the AABB of a box rotated and scaled by a node transform.
/// A box centered on its pivot stays centered under the transform, so only the
/// half-extent picks up the rotation: `sum_j abs(col_j) * half[j]` over the
/// rotated, scaled basis columns.
fn transform_half(transform: &TyTransformF64, half: [f64; 3]) -> [f64; 3] {
    let col_x = transform
        .rotation
        .rotate(TyVector3F64::new(transform.scale.x, 0.0, 0.0));
    let col_y = transform
        .rotation
        .rotate(TyVector3F64::new(0.0, transform.scale.y, 0.0));
    let col_z = transform
        .rotation
        .rotate(TyVector3F64::new(0.0, 0.0, transform.scale.z));
    [
        col_x.x.abs() * half[0] + col_y.x.abs() * half[1] + col_z.x.abs() * half[2],
        col_x.y.abs() * half[0] + col_y.y.abs() * half[1] + col_z.y.abs() * half[2],
        col_x.z.abs() * half[0] + col_y.z.abs() * half[1] + col_z.z.abs() * half[2],
    ]
}

/// Builds a scene group from its node and preserved provenance. The content box
/// `(center, half)` is the group's derived subtree box, written as the
/// symmetric `e_c`/`e_mi`/`e_ma` Voxel Max stores.
fn group_from_node(
    node: &VoxHierarchyNode,
    ext_node: &VoxelMaxNode,
    ind: [i64; 3],
    center: [f64; 3],
    half: [f64; 3],
) -> VMaxGroup {
    VMaxGroup {
        name: node.name.clone(),
        id: ext_node.id.clone(),
        parent_id: ext_node.parent_id.clone(),
        hidden: None,
        position: vector(node.transform.position),
        rotation: ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE),
        scale: vector(node.transform.scale),
        ind,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone().unwrap_or_default(),
        t_pa: ext_node.pivot_align.clone().unwrap_or_default(),
        t_pf: ext_node.pivot_face.clone().unwrap_or_default(),
        t_po: None,
        center,
        bounds_min: Some([-half[0], -half[1], -half[2]]),
        bounds_max: Some(half),
    }
}

/// The `ind` path triple for an emitted node: the preserved one from the ext,
/// or a synthesized triple keeping every node distinct, since Voxel Max
/// collapses nodes that share `[0, 0, 0]`. Groups take the `1` lane and objects
/// the `0` lane.
fn node_ind(ext_node: &VoxelMaxNode, is_group: bool, counter: &mut i64) -> [i64; 3] {
    if let Some(index) = ext_node.index {
        return index;
    }
    let ind = [0, i64::from(is_group), *counter];
    *counter += 1;
    ind
}

/// The node's rotation as a quaternion, decoded from the stored axis-angle like
/// the read path so the two stay inverses. A degenerate axis decodes to
/// identity.
fn ext_rotation(ext_node: &VoxelMaxNode) -> TyQuaternionF64 {
    let [x, y, z, angle] = ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE);
    TyQuaternionF64::from_axis_angle(TyVector3F64::new(x, y, z), angle)
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

/// Recovers an object's `t_p`, the inverse of the read path's
/// `object_transform`. It backs out the `t_p` Voxel Max renders with from the
/// node's transform, the content center it pivots about, and the grid `origin`:
/// `t_p = position - center - R*S* (box_min - center - origin)`. Uses the
/// stored axis-angle `rotation`, not the live transform's, so it stays an exact
/// inverse when synthesis drops a node's rotation to identity.
fn unbake_position(
    transform: &TyTransformF64,
    rotation: TyQuaternionF64,
    placement: &ObjectPlacement,
) -> [f64; 3] {
    let center = placement.center;
    let box_min = placement.box_min;
    let origin = placement.origin;
    let scale = transform.scale;
    let offset = TyVector3F64::new(
        (box_min[0] as f64 - center[0] - origin[0] as f64) * scale.x,
        (box_min[1] as f64 - center[1] - origin[1] as f64) * scale.y,
        (box_min[2] as f64 - center[2] - origin[2] as f64) * scale.z,
    );
    let rotated = rotation.rotate(offset);
    [
        transform.position.x - center[0] - rotated.x,
        transform.position.y - center[1] - rotated.y,
        transform.position.z - center[2] - rotated.z,
    ]
}

/// A `[f64; 3]` from a vector.
fn vector(vector: TyVector3F64) -> [f64; 3] {
    [vector.x, vector.y, vector.z]
}

/// A syntactically valid, deterministic UUID for a synthesized scene node.
/// Voxel Max decodes a node's `id`/`pid` as a UUID and rejects a non-UUID
/// token. The index is offset by one so the first node avoids the all-zero nil
/// UUID.
fn synth_uuid(index: usize) -> String {
    format!("00000000-0000-0000-0000-{:012X}", index + 1)
}

/// A distinct, valid UUID for an extra object on a node placing several,
/// stamping the object's slot into the node id's fourth group. A node id keeps
/// that group zero, so this never collides with a node or another slot.
fn secondary_uuid(node_id: &str, slot: usize) -> String {
    match node_id.split('-').collect::<Vec<_>>().as_slice() {
        [a, b, c, _, e] => format!("{a}-{b}-{c}-{slot:04X}-{e}"),
        _ => node_id.to_owned(),
    }
}

/// The filename suffix for object `index`: empty for the first, then the index.
fn suffix(index: usize) -> String {
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}
