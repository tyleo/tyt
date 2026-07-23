use crate::{
    ABSORPTION, BASE_COLOR_FACTOR, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, Error, IOR, METALLIC_FACTOR,
    ROUGHNESS_FACTOR, Result, SHADOWS, SceneCameraSource, TRANSMISSION_FACTOR, VoxelMaxColorFormat,
    VoxelMaxExt, VoxelMaxExtWrapper, VoxelMaxMaterial, VoxelMaxNode, VoxelMaxPalette, ext_for,
    from_vox_value, pbr_factor_to_vm_coefficient, pool_color, tighten,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use ty_math::{
    TyBoundsF64, TyLinSrgbaF64, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64,
    TyVector3I32, TyVector3U32,
};
use vmax::{
    VMaxBrush, VMaxBrushColor, VMaxBrushEntry, VMaxBrushState, VMaxCamera, VMaxContentsVmaxbFile,
    VMaxFile, VMaxFlag, VMaxFlagValue, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxMode,
    VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
    VMaxSceneJsonFile, VMaxToolMode, VMaxTools, VMaxViewBox,
};
use vmax_codec::{VMaxVoxel, encode_vmax_snapshots};
use voxcore::{
    BVoxArrayProperty, BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxPoolValue,
    VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
};

/// Usable colors in a Voxel Max palette. Color indices are 1-based: `color_idx`
/// is `cell + 1`, runs 1..=255, and 0 is the empty cell. Colors are stored
/// 0-based; a `palette*.png` appends a transparent terminator (256 entries),
/// the plist `colors` table does not (255 entries).
const PALETTE_COLORS: usize = 255;

/// The material slots every Voxel Max palette carries. A color cell's material
/// is a bit in the settings `lc` byte, so at most 8 (0..=7) fit; the sidecar
/// always lists exactly this many, real materials in the low slots and the rest
/// padded with the neutral default.
const MATERIAL_SLOTS: usize = 8;

/// The neutral default material Voxel Max fills unused slots with: matte, not
/// metallic, shadow-casting.
const DEFAULT_METALLIC: f64 = 0.1;
const DEFAULT_ROUGHNESS: f64 = 0.9;

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
const FALLBACK_PALETTE: &str = "palette1.png";

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
    let mut palette_files: HashMap<U32Id<BVoxPalette>, String> = HashMap::new();
    // Object id -> its `data` filename, shared by every node that places it.
    let mut contents_by_object: HashMap<U32Id<BVoxObject>, String> = HashMap::new();

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
        for (slot, object_id) in node.child_objects.iter().enumerate() {
            let object_id = *object_id;
            let object = state.object(object_id).expect("a valid node child object");
            let folded = folded_ref(state, object);
            let plan = match folded.as_ref() {
                Some(folded) => material_plan(state, folded, &voxel_max.palettes)?,
                None => MaterialPlan::default(),
            };
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
                .get(object_id.to_u32() as usize)
                .and_then(|s| s.clone());

            // Instances share one contents file: rebuild it once.
            let data = match contents_by_object.get(&object_id) {
                Some(data) => data.clone(),
                None => {
                    let voxels = reconstruct_voxels(
                        state,
                        &tight,
                        folded.as_ref(),
                        &plan,
                        placement.box_min,
                    )?;
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
                folded.as_ref(),
                &plan,
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
    let box_local =
        TyBoundsF64::from_min_size(TyVector3I32::from_array(box_min).to_f64(), bounds.to_f64());
    (
        box_local.center.to_array(),
        (-box_local.extents).to_array(),
        box_local.extents.to_array(),
    )
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

/// The folded palette an object references: the palette id, the optional
/// `baseColorFactor` array property, and the material array properties in
/// order.
struct FoldedRef {
    palette: U32Id<BVoxPalette>,
    color: Option<U32Id<BVoxArrayProperty>>,
    materials: Vec<(String, U32Id<BVoxArrayProperty>)>,
}

/// The one folded palette an object references on its single layer, or `None`
/// when it references no layer.
fn folded_ref(state: &VoxMain, object: &VoxObject) -> Option<FoldedRef> {
    let (_, palette_id) = object.iter_layers().next()?;
    let palette = state.palette(palette_id)?;
    let color = palette.array_property_by_name(BASE_COLOR_FACTOR);
    let materials = palette
        .iter_array_properties()
        .filter(|(id, _)| Some(*id) != color)
        .map(|(id, property)| (property.name.clone(), id))
        .collect();
    Some(FoldedRef {
        palette: palette_id,
        color,
        materials,
    })
}

/// The material reconstruction for a folded palette: each folded material's
/// Voxel Max `material_idx`, the exact material list for the settings sidecar,
/// and the sidecar display name.
#[derive(Default)]
struct MaterialPlan {
    name: String,
    material_idx: HashMap<U32Id<BVoxMaterial>, u8>,
    materials: Vec<VMaxMaterial>,
}

/// Reconstructs the Voxel Max materials for a folded palette. A Voxel-Max-origin
/// state carries the exact list in its ext, whose pools hold one value per
/// material so a material's index is its value id into the first material
/// property; a state loaded from another format has no such list, so the
/// materials are derived from the pools, one per distinct material signature.
fn material_plan(
    state: &VoxMain,
    folded: &FoldedRef,
    ext_palettes: &[Option<VoxelMaxPalette>],
) -> Result<MaterialPlan> {
    let provenance = ext_palettes
        .get(folded.palette.to_u32() as usize)
        .and_then(|palette| palette.as_ref());
    let name = provenance
        .map(|palette| palette.name.clone())
        .unwrap_or_default();
    let palette = state.palette(folded.palette).expect("a referenced palette");

    // A color-only palette folds no materials; every material writes index 0.
    if folded.materials.is_empty() {
        let material_idx = palette
            .iter_materials()
            .map(|material| (material, 0))
            .collect();
        return Ok(MaterialPlan {
            name,
            material_idx,
            materials: Vec::new(),
        });
    }

    // A Voxel-Max-origin list came from a real document, so it is within budget:
    // the 0-based value id is the material byte, below the material count. A
    // hand-edited ext can still exceed the single-byte budget, so it is checked.
    if let Some(provenance) = provenance.filter(|palette| !palette.materials.is_empty()) {
        let first = folded.materials[0].1;
        let mut material_idx = HashMap::new();
        for material in palette.iter_materials() {
            let index = palette
                .value_id(material, first)
                .map_or(0, |id| id.to_u32());
            if index >= MATERIAL_SLOTS as u32 {
                return Err(Error::invalid(format!(
                    "a voxel references material {index}, but a Voxel Max palette holds only \
                     {MATERIAL_SLOTS} material slots"
                )));
            }
            material_idx.insert(material, index as u8);
        }
        let materials = provenance
            .materials
            .iter()
            .enumerate()
            .map(|(slot, material)| vmax_material(slot, material))
            .collect();
        return Ok(MaterialPlan {
            name,
            material_idx,
            materials,
        });
    }

    derive_materials(state, folded, palette, name)
}

/// Derives a Voxel Max material per distinct signature of the material
/// properties, for a state that carries no exact material list. The signature
/// index is the `material_idx`, and each material reads its coefficients from
/// the pools. Errors when the distinct materials exceed [`MATERIAL_SLOTS`],
/// since a Voxel Max palette holds only that many, so a cross-format source
/// with too many materials cannot be represented rather than silently
/// wrapping.
fn derive_materials(
    state: &VoxMain,
    folded: &FoldedRef,
    palette: &VoxPalette,
    name: String,
) -> Result<MaterialPlan> {
    // The linear luminance of a material's base color, the reference Voxel Max
    // glows against, or `None` when the palette carries no base color.
    let base_luminance = |material| -> Option<f64> {
        let color = folded.color?;
        let value_id = palette.value_id(material, color)?;
        let pool = array_property_pool(state, folded.palette, color)?;
        let [r, g, b, _] = pool_color(pool, value_id)?;
        let linear: TyLinSrgbaF64 = TySrgbaU8::from([r, g, b, 255])
            .into_format::<f64, f64>()
            .into_linear();
        Some(0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue)
    };

    let mut signatures: Vec<Vec<U32Id<BVoxPoolValue>>> = Vec::new();
    // Parallel to `signatures`: the base luminance of the first material to claim
    // each slot, the reference its emissive is read against.
    let mut base_luminances: Vec<Option<f64>> = Vec::new();
    let mut index_of: HashMap<Vec<U32Id<BVoxPoolValue>>, u8> = HashMap::new();
    let mut material_idx = HashMap::new();
    for material in palette.iter_materials() {
        let signature: Vec<U32Id<BVoxPoolValue>> = folded
            .materials
            .iter()
            .map(|(_, array_property)| {
                palette
                    .value_id(material, *array_property)
                    .unwrap_or(U32Id::from_u32(0))
            })
            .collect();
        let idx = match index_of.get(&signature) {
            Some(&idx) => idx,
            None => {
                if signatures.len() >= MATERIAL_SLOTS {
                    return Err(Error::invalid(format!(
                        "an object needs more than {MATERIAL_SLOTS} materials, but a Voxel Max \
                         palette holds only that many material slots"
                    )));
                }
                let idx = signatures.len() as u8;
                signatures.push(signature.clone());
                base_luminances.push(base_luminance(material));
                index_of.insert(signature, idx);
                idx
            }
        };
        material_idx.insert(material, idx);
    }
    let materials = signatures
        .iter()
        .enumerate()
        .map(|(slot, signature)| {
            derived_material(
                state,
                folded.palette,
                &folded.materials,
                slot,
                signature,
                base_luminances[slot],
            )
        })
        .collect();
    Ok(MaterialPlan {
        name,
        material_idx,
        materials,
    })
}

/// One derived Voxel Max material. A coefficient reads from its array
/// property's pool at the signature's value id, or from the palette's scalar
/// property of the same name when no column carries it. Metalness and
/// roughness map from the 0 to 1 glTF factor to Voxel Max's 0.1 to 0.9 slider
/// coefficient; see [`pbr_factor_to_vm_coefficient`].
fn derived_material(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    properties: &[(String, U32Id<BVoxArrayProperty>)],
    slot: usize,
    signature: &[U32Id<BVoxPoolValue>],
    base_luminance: Option<f64>,
) -> VMaxMaterial {
    let scalar = |property: &str| -> Option<f64> {
        match properties.iter().position(|(name, _)| name == property) {
            Some(position) => {
                pool_scalar(state, palette, properties[position].1, signature[position])
            }
            None => scalar_property_scalar(state, palette, property),
        }
    };
    let flag = |property: &str| -> Option<bool> {
        match properties.iter().position(|(name, _)| name == property) {
            Some(position) => {
                pool_flag(state, palette, properties[position].1, signature[position])
            }
            None => scalar_property_flag(state, palette, property),
        }
    };
    // The emissive color's linear luminance at this slot, or `None` when the
    // property is absent. Folded into Voxel Max's single self-illumination
    // coefficient, read relative to the base color the caller supplies.
    let emissive_luminance = || -> Option<f64> {
        let position = properties
            .iter()
            .position(|(name, _)| name == EMISSIVE_FACTOR)?;
        let pool = array_property_pool(state, palette, properties[position].1)?;
        let [r, g, b, _] = pool_color(pool, signature[position])?;
        let linear: TyLinSrgbaF64 = TySrgbaU8::from([r, g, b, 255])
            .into_format::<f64, f64>()
            .into_linear();
        Some(0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue)
    };
    let carries = |property: &str| -> bool {
        properties.iter().any(|(name, _)| name == property)
            || state
                .palette(palette)
                .is_some_and(|palette| palette.scalar_property_by_name(property).is_some())
    };
    let dispersed = carries(IOR) || carries(TRANSMISSION_FACTOR) || carries(ABSORPTION);
    VMaxMaterial {
        mi: (slot + 1).to_string(),
        mc: pbr_factor_to_vm_coefficient(scalar(METALLIC_FACTOR).unwrap_or(0.0)),
        rc: pbr_factor_to_vm_coefficient(scalar(ROUGHNESS_FACTOR).unwrap_or(0.0)),
        // Voxel Max glows in the voxel's base color at coefficient `sic`, so
        // the emissive folds to its luminance relative to the base color, times
        // `emissiveStrength`. This inverts the from-vmax split, which emits the
        // base color as the emissive color and `sic` as the strength. `sic` is
        // unbounded; Voxel Max's 0 to 100 slider is `sic` 0 to 20. A black
        // factor stays matte; a missing or black base color cannot normalize, so
        // the bare emissive luminance stands in, and with no emissive color the
        // strength stands alone.
        sic: match emissive_luminance() {
            Some(emissive) => {
                let strength = scalar(EMISSIVE_STRENGTH).unwrap_or(1.0);
                match base_luminance {
                    Some(base) if base > 0.0 => strength * emissive / base,
                    _ => strength * emissive,
                }
            }
            None => scalar(EMISSIVE_STRENGTH).unwrap_or(0.0),
        },
        // Voxel Max casts shadows by default; a source without a shadows flag,
        // such as glTF, takes that default.
        sh: flag(SHADOWS).unwrap_or(true),
        tc: None,
        md: dispersed.then(|| VMaxMaterialDispersion {
            absorption: scalar(ABSORPTION).unwrap_or(0.0),
            ior: scalar(IOR).unwrap_or(1.5),
            transmission: scalar(TRANSMISSION_FACTOR).unwrap_or(0.0),
        }),
    }
}

/// Rebuilds a Voxel Max material from its exact ext copy. The `mi` token is
/// derived from the 1-based slot and the transparency color `tc` is dropped,
/// matching the writer's behavior.
fn vmax_material(slot: usize, material: &VoxelMaxMaterial) -> VMaxMaterial {
    VMaxMaterial {
        mi: (slot + 1).to_string(),
        mc: material.metallic,
        rc: material.roughness,
        sic: material.emissive,
        sh: material.shadows,
        tc: material.transmission_color,
        md: material
            .dispersion
            .as_ref()
            .map(|dispersion| VMaxMaterialDispersion {
                absorption: dispersion.absorption,
                ior: dispersion.ior,
                transmission: dispersion.transmission,
            }),
    }
}

/// The `f64` value at `value_id` in an array property's `float` pool, or
/// `None`.
fn pool_scalar(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    array_property: U32Id<BVoxArrayProperty>,
    value_id: U32Id<BVoxPoolValue>,
) -> Option<f64> {
    match array_property_pool(state, palette, array_property)? {
        VoxValuePool::Float { values, .. } => values.get(value_id.to_usize_id()).copied(),
        _ => None,
    }
}

/// The `bool` value at `value_id` in an array property's `bool` pool, or
/// `None`.
fn pool_flag(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    array_property: U32Id<BVoxArrayProperty>,
    value_id: U32Id<BVoxPoolValue>,
) -> Option<bool> {
    match array_property_pool(state, palette, array_property)? {
        VoxValuePool::Bool { values } => values.get(value_id.to_usize_id()).copied(),
        _ => None,
    }
}

/// The `f64` value the palette's scalar property `name` pins in a `float`
/// pool, or `None`.
fn scalar_property_scalar(state: &VoxMain, palette: U32Id<BVoxPalette>, name: &str) -> Option<f64> {
    let property = state.palette(palette)?.scalar_property_by_name(name)?;
    match state.scalar_property_value(palette, property)? {
        (VoxValuePool::Float { values, .. }, value_id) => {
            values.get(value_id.to_usize_id()).copied()
        }
        _ => None,
    }
}

/// The `bool` value the palette's scalar property `name` pins in a `bool`
/// pool, or `None`.
fn scalar_property_flag(state: &VoxMain, palette: U32Id<BVoxPalette>, name: &str) -> Option<bool> {
    let property = state.palette(palette)?.scalar_property_by_name(name)?;
    match state.scalar_property_value(palette, property)? {
        (VoxValuePool::Bool { values }, value_id) => values.get(value_id.to_usize_id()).copied(),
        _ => None,
    }
}

/// The value pool an array property draws from.
fn array_property_pool(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    array_property: U32Id<BVoxArrayProperty>,
) -> Option<&VoxValuePool> {
    let pool = state.palette(palette)?.array_property(array_property)?.pool;
    state.value_pool(pool)
}

/// Re-bases the tight object's voxels to absolute model space, recovering each
/// one's `color_idx` from its material's `baseColorFactor` value id and its
/// `material_idx` from the material plan. A colorless voxel takes index 1.
fn reconstruct_voxels(
    state: &VoxMain,
    object: &VoxObject,
    folded: Option<&FoldedRef>,
    plan: &MaterialPlan,
    box_min: [i32; 3],
) -> Result<Vec<VMaxVoxel>> {
    let layer = object.iter_layers().next().map(|(layer, _)| layer);
    object
        .iter_live()
        .map(|voxel| {
            let position = object
                .voxel_position(voxel)
                .expect("a live voxel is within the grid");
            let material = layer.and_then(|layer| object.voxel_material(voxel, layer));
            let color_idx = match (folded, material) {
                (Some(folded), Some(material)) => color_index(state, folded, material)?,
                // A colorless voxel still needs a non-empty index, so it takes 1,
                // not the empty index 0.
                _ => 1,
            };
            // Voxel Max's material byte is 0-based: byte `n` selects
            // `materials[n]`. The per-color material map in the sidecar (`lc`)
            // drives what renders, but the byte is kept consistent with it.
            let material_idx = material
                .and_then(|material| plan.material_idx.get(&material).copied())
                .unwrap_or(0);
            Ok(VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx,
                color_idx,
            })
        })
        .collect()
}

/// The 1-based Voxel Max color index a `material` samples through
/// `baseColorFactor`. Errors when the color value id reaches
/// [`PALETTE_COLORS`], one past the last usable color, so a padded source
/// palette is fine as long as its referenced colors fit.
fn color_index(state: &VoxMain, folded: &FoldedRef, material: U32Id<BVoxMaterial>) -> Result<u8> {
    let Some(color) = folded.color else {
        return Ok(1);
    };
    let index = state
        .palette(folded.palette)
        .and_then(|palette| palette.value_id(material, color))
        .map_or(0, |id| id.to_u32());
    if index >= PALETTE_COLORS as u32 {
        return Err(Error::invalid(format!(
            "a voxel references color cell {index}, but a Voxel Max palette holds only \
             {PALETTE_COLORS} colors, so the source has more colors than fit"
        )));
    }
    Ok(index as u8 + 1)
}

/// Returns the `pal` filename for an object, building its color image and
/// material sidecar the first time the folded palette is seen. An object with no
/// color property borrows the default palette name and writes no file.
#[allow(clippy::too_many_arguments)]
fn build_palette(
    state: &VoxMain,
    folded: Option<&FoldedRef>,
    plan: &MaterialPlan,
    palette_files: &mut HashMap<U32Id<BVoxPalette>, String>,
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    voxel_max_color_format: VoxelMaxColorFormat,
) -> String {
    // An object with no color property borrows the default palette name; an empty
    // reference is one Voxel Max cannot resolve. No file is written for it.
    let Some((palette_id, color)) = folded.and_then(|folded| Some((folded.palette, folded.color?)))
    else {
        return FALLBACK_PALETTE.to_owned();
    };
    if let Some(name) = palette_files.get(&palette_id) {
        return name.clone();
    }
    // Voxel Max numbers palette files 1-based (`palette1`, `palette2`, ...); an
    // un-numbered `palette.png` breaks the plist color lookup when no image is
    // written.
    let stem = palette_files.len() + 1;
    let pal = format!("palette{stem}.png");

    let colors = color_palette_colors(state, palette_id, color);
    if matches!(
        voxel_max_color_format,
        VoxelMaxColorFormat::Png | VoxelMaxColorFormat::All
    ) {
        // 256 entries: the 255 colors 0-based then a transparent terminator.
        let mut cells = colors.clone();
        cells.push([0, 0, 0, 0]);
        palette_png_files.insert(pal.clone(), VMaxPalettePngFile(cells));
    }
    // The settings sidecar carries the materials, and the colors when no image
    // does. Plist mode writes no image, so even a color-only object writes its
    // colors here rather than dropping them.
    let write_sidecar =
        !plan.materials.is_empty() || matches!(voxel_max_color_format, VoxelMaxColorFormat::Plist);
    if write_sidecar {
        let sidecar = format!("palette{stem}.settings.vmaxpsb");
        // The plist `colors` table is the 255 colors with no terminator.
        let sidecar_colors = match voxel_max_color_format {
            VoxelMaxColorFormat::Png => Vec::new(),
            VoxelMaxColorFormat::Plist | VoxelMaxColorFormat::All => colors,
        };
        // The per-color material map Voxel Max renders from: each used color
        // cell carries a bit for the material it draws.
        let (lc, indices, current) = color_material_map(state, palette_id, color, plan);
        palette_settings_files.insert(
            sidecar,
            material_settings(
                material_name(&plan.name),
                plan.materials.clone(),
                sidecar_colors,
                lc,
                indices,
                current,
            ),
        );
    }
    palette_files.insert(palette_id, pal.clone());
    pal
}

/// The color array property's pool decoded to exactly [`PALETTE_COLORS`]
/// 0-based RGBA entries, padded with transparent entries or truncated to that
/// count. Colors past the budget are dropped; a voxel that would reference one
/// is rejected by [`reconstruct_voxels`].
fn color_palette_colors(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    color: U32Id<BVoxArrayProperty>,
) -> Vec<[u8; 4]> {
    let mut cells: Vec<[u8; 4]> = Vec::new();
    if let Some(pool) = array_property_pool(state, palette, color) {
        for index in 0..pool.values_len().min(PALETTE_COLORS) {
            cells.push(pool_color(pool, U32Id::from_u32(index as u32)).unwrap_or([0, 0, 0, 0]));
        }
    }
    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    cells
}

/// Builds a settings sidecar carrying `colors`, `materials`, and the per-color
/// material map (`lc`/`indices`/`current`). The materials are padded to the
/// fixed slot count and every coefficient f32-rounded, since Voxel Max drops a
/// palette whose material coefficients are not f32-representable. The remaining
/// editor-state keys are filled with the defaults Voxel Max expects.
fn material_settings(
    name: String,
    materials: Vec<VMaxMaterial>,
    colors: Vec<[u8; 4]>,
    lc: Vec<u8>,
    indices: Vec<i64>,
    current: i64,
) -> VMaxPaletteSettingsVmaxpsbFile {
    VMaxPaletteSettingsVmaxpsbFile {
        name,
        materials: pad_materials(materials),
        colors: colors.iter().flatten().copied().collect(),
        indices,
        lc,
        palette_type: 0,
        transparency: 1.0,
        r: 0,
        rt: "n".to_owned(),
        cmt: "ng".to_owned(),
        current,
        ali: "1".to_owned(),
        voxmats: Vec::new(),
        ls: Vec::new(),
    }
}

/// The palette display name Voxel Max shows: the preserved name, or its default
/// when a synthesized palette has none.
fn material_name(name: &str) -> String {
    if name.is_empty() {
        "Palette #1".to_owned()
    } else {
        name.to_owned()
    }
}

/// F32-rounds every material's coefficients and, for a palette that carries
/// materials, pads the list to [`MATERIAL_SLOTS`] with the neutral default. A
/// color-only palette keeps an empty list, since Voxel Max then uses its own
/// default materials.
fn pad_materials(materials: Vec<VMaxMaterial>) -> Vec<VMaxMaterial> {
    if materials.is_empty() {
        return materials;
    }
    let mut out: Vec<VMaxMaterial> = materials.iter().map(f32_material).collect();
    while out.len() < MATERIAL_SLOTS {
        out.push(default_material(out.len()));
    }
    out
}

/// A coefficient snapped to an f32-exact value. Voxel Max decodes material
/// coefficients as 32-bit floats and drops a whole palette whose coefficients
/// are not f32-representable, so every one passes through here.
fn to_f32(value: f64) -> f64 {
    f64::from(value as f32)
}

/// A copy of `material` with every coefficient f32-rounded by [`to_f32`].
fn f32_material(material: &VMaxMaterial) -> VMaxMaterial {
    VMaxMaterial {
        mi: material.mi.clone(),
        mc: to_f32(material.mc),
        rc: to_f32(material.rc),
        sic: to_f32(material.sic),
        sh: material.sh,
        tc: material.tc.map(to_f32),
        md: material
            .md
            .as_ref()
            .map(|dispersion| VMaxMaterialDispersion {
                absorption: to_f32(dispersion.absorption),
                ior: to_f32(dispersion.ior),
                transmission: to_f32(dispersion.transmission),
            }),
    }
}

/// The neutral default material Voxel Max fills the slot at `slot` with.
fn default_material(slot: usize) -> VMaxMaterial {
    VMaxMaterial {
        mi: (slot + 1).to_string(),
        mc: to_f32(DEFAULT_METALLIC),
        rc: to_f32(DEFAULT_ROUGHNESS),
        sic: 0.0,
        sh: true,
        tc: None,
        md: None,
    }
}

/// The per-color material map for a palette's settings sidecar. For each color
/// cell a material draws, sets that material's bit in `lc` (`1 <<
/// material_byte`), lists the used cells in `indices`, and takes the first as
/// `current`. Empty for a palette with no materials, which Voxel Max renders
/// with its own defaults. Voxel Max reads a voxel's material from this map, not
/// the per-voxel byte.
fn color_material_map(
    state: &VoxMain,
    palette: U32Id<BVoxPalette>,
    color: U32Id<BVoxArrayProperty>,
    plan: &MaterialPlan,
) -> (Vec<u8>, Vec<i64>, i64) {
    let mut lc = vec![0u8; 256];
    if plan.materials.is_empty() {
        return (lc, Vec::new(), 0);
    }
    let mut cells: BTreeSet<u32> = BTreeSet::new();
    if let Some(palette_ref) = state.palette(palette) {
        for material in palette_ref.iter_materials() {
            let Some(cell) = palette_ref.value_id(material, color).map(|id| id.to_u32()) else {
                continue;
            };
            let Some(&byte) = plan.material_idx.get(&material) else {
                continue;
            };
            if let Some(slot) = lc.get_mut(cell as usize) {
                *slot |= 1 << byte;
                cells.insert(cell);
            }
        }
    }
    let indices: Vec<i64> = cells.iter().map(|&cell| i64::from(cell)).collect();
    let current = indices.first().copied().unwrap_or(0);
    (lc, indices, current)
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
        scale: node.transform.scale.to_array(),
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
        let center = transform
            .transform_point(TyVector3F64::from_array(child_center))
            .to_array();
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
    let box_local = if bounds.x == 0 && bounds.y == 0 && bounds.z == 0 {
        TyBoundsF64::from_min_size(edit_origin.to_f64(), edit_bounds.to_f64())
    } else {
        TyBoundsF64::from_min_size(tight.origin().to_f64(), bounds.to_f64())
    };
    (box_local.center.to_array(), box_local.extents.to_array())
}

/// Grows the running `(min, max)` AABB to include the box centered at `center`
/// with half-extents `half`.
fn extend_bounds(bounds: &mut Option<([f64; 3], [f64; 3])>, center: [f64; 3], half: [f64; 3]) {
    let center = TyVector3F64::from_array(center);
    let half = TyVector3F64::from_array(half);
    let lo = center - half;
    let hi = center + half;
    match bounds {
        Some((min, max)) => {
            *min = TyVector3F64::from_array(*min)
                .component_min_with(&lo)
                .to_array();
            *max = TyVector3F64::from_array(*max)
                .component_max_with(&hi)
                .to_array();
        }
        None => *bounds = Some((lo.to_array(), hi.to_array())),
    }
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
        position: node.transform.position.to_array(),
        rotation: ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE),
        scale: node.transform.scale.to_array(),
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

/// The filename suffix for an object: empty for object 0, then its numeric id.
fn suffix(object: U32Id<BVoxObject>) -> String {
    let index = object.to_u32();
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}
