use crate::{
    Error, Result, VoxelMaxColorFormat, VoxelMaxExt, VoxelMaxExtWrapper, VoxelMaxNode,
    VoxelMaxPalette, ext_for, from_vox_value,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, HashMap};
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};
use vmax::{
    VMaxBrush, VMaxBrushColor, VMaxBrushEntry, VMaxBrushState, VMaxCamera, VMaxContentsVmaxbFile,
    VMaxFile, VMaxFlag, VMaxFlagValue, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxMode,
    VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
    VMaxSceneJsonFile, VMaxToolMode, VMaxTools, VMaxViewBox,
};
use vmax_codec::{VMaxVoxel, encode_vmax_snapshots};
use voxcore::{
    BVoxHierarchyNode, BVoxPalette, BVoxPaletteRef, VoxHierarchyNode, VoxObject, VoxPalette,
    VoxState, VoxValue,
};

/// Usable colors in a Voxel Max palette: indices 0..254. Index 255 is a reserved
/// transparent terminator no voxel references, so the table holds this many
/// entries and the image pads to 256 by appending the terminator.
const PALETTE_COLORS: usize = 255;

/// Codable version stamped on a rebuilt contents file when the state carries no
/// preserved object version.
const FALLBACK_CONTENT_VERSION: i64 = 4;

/// Axis-angle stored on a node with no preserved rotation; a degenerate axis
/// decodes to the identity quaternion.
const IDENTITY_AXIS_ANGLE: [f64; 4] = [0.0, 0.0, 0.0, 0.0];

/// Default transform-anchor tokens for a synthesized node. Voxel Max decodes each
/// as an enum and rejects an empty token.
const DEFAULT_ALIGNMENT: &str = "f";
const DEFAULT_PIVOT_ALIGN: &str = "4";
const DEFAULT_PIVOT_FACE: &str = "8";

/// The `pal` an object with no color palette borrows. An empty reference makes
/// Voxel Max read the package directory as a file and abort, so a colorless object
/// shares the first color palette's name and writes no file of its own.
const FALLBACK_PALETTE: &str = "palette.png";

/// The scene camera a synthesized document opens with, mirroring a fresh Voxel Max
/// document's neutral rig. Voxel Max needs a valid rig to present an imported
/// document; the framing is cosmetic.
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

/// Default `tools` for a synthesized object. Voxel Max's object decoder rejects a
/// sparse `tools`, so this mirrors a fresh object's editor state.
fn default_tools() -> VMaxTools {
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
        vp: Some(VMaxViewBox {
            min: [0, 0, 0],
            max: [255, 255, 255],
        }),
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

/// Default `brush` palette for a synthesized object, mirroring a fresh document.
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

/// Default editor `cam` for a synthesized object, a neutral view at the origin.
fn default_camera() -> VMaxCamera {
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
        o: [0.0, 0.0, 0.0],
    }
}

/// Writes a [`VoxState`] back to a Voxel Max document, the inverse of
/// [`from_vmax_file`](crate::from_vmax_file).
/// `voxel_max_color_format` selects where each palette's colors are stored, as
/// described on [`VoxelMaxColorFormat`].
///
/// A state carrying the `voxel-max` ext the forward path writes is rebuilt from
/// it exactly, and editor session artifacts voxcore does not model are dropped. A
/// state without that ext, such as one loaded from another format, has its ext
/// synthesized from the bare voxcore scene by [`synthesize_voxel_max_ext`] and the
/// rest of this path runs unchanged.
pub fn to_vmax_file(
    state: &VoxState,
    voxel_max_color_format: VoxelMaxColorFormat,
) -> Result<VMaxFile> {
    let (voxel_max, color_offset, placements) = match ext_for(state, "voxel-max") {
        Some(ext) => {
            let voxel_max = from_vox_value::<VoxelMaxExtWrapper>(ext)?.voxel_max;
            let placements = ext_placements(state, &voxel_max);
            (voxel_max, 0, placements)
        }
        // Voxel Max reads color index 0 as an empty cell, so a synthesized palette
        // reserves index 0 and shifts every real color up by one. A voxcore palette
        // built from another format uses cell 0 as a real color, which would
        // otherwise drop voxels that reference it. The lossless ext path already
        // satisfies this convention, so it keeps the zero offset.
        None => {
            let voxel_max = synthesize_voxel_max_ext(state);
            let placements = synthesize_placements(state);
            (voxel_max, 1, placements)
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

    for placement in &placements {
        let node = placement.node;
        let ext_node = &placement.ext;

        if node.child_objects.is_empty() {
            let ind = node_ind(ext_node, true, &mut ind_counter);
            groups.push(group_from_node(node, ext_node, ind));
            continue;
        }

        // One scene object per child object. A vmax-origin node always carries a
        // single object, so this loops once and matches the lossless path exactly.
        // A synthesized node may carry several, such as a Goxel layer's blocks; the
        // extra objects become sibling object-nodes under the same parent.
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
                secondary_object_ext(ext_node, object, slot)
            };

            // Instances share one contents file: rebuild it once.
            let data = match contents_by_object.get(&object_id) {
                Some(data) => data.clone(),
                None => {
                    let voxels =
                        reconstruct_voxels(object, color, material, &object_ext, color_offset)?;
                    let data = format!("contents{suffix}.vmaxb");
                    // Voxels re-encode into snapshots; serde-only state the decoded
                    // voxcore object does not model (`pal`) stays absent.
                    let contents = match voxel_max
                        .object_states
                        .get(object_id)
                        .and_then(|s| s.clone())
                    {
                        Some(state) => VMaxContentsVmaxbFile {
                            snapshots: encode_vmax_snapshots(&voxels),
                            uuid: state.uuid,
                            v: state.v,
                            tools: state.tools,
                            brush: state.brush,
                            cam: state.cam,
                            pal: None,
                        },
                        // Voxel Max's object decoder rejects a sparse editor state,
                        // so a synthesized object carries default `tools`, `brush`,
                        // and `cam`.
                        None => VMaxContentsVmaxbFile {
                            snapshots: encode_vmax_snapshots(&voxels),
                            uuid: object_ext.id.clone(),
                            v: FALLBACK_CONTENT_VERSION,
                            tools: Some(default_tools()),
                            brush: Some(default_brush()),
                            cam: Some(default_camera()),
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
                color_offset,
            );

            let ind = node_ind(&object_ext, false, &mut ind_counter);
            objects.push(object_from_node(node, &object_ext, data, pal, &suffix, ind));
        }
    }

    let mut scene = voxel_max.scene;
    scene.groups = groups;
    scene.objects = objects;

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

/// One scene node to emit and the Voxel Max provenance that places it: the
/// voxcore node supplies the local transform, the ext supplies the id, parent, and
/// bounds. The lossless path pairs each voxcore node with its ext entry by index;
/// synthesis walks the hierarchy from the roots, so a node shared by several
/// parents, or one that is both a root and a child, is emitted once per path the
/// way voxcore renders it, and a node reachable from no root is dropped just as
/// voxcore never places it.
struct Placement<'a> {
    node: &'a VoxHierarchyNode,
    ext: VoxelMaxNode,
}

/// Pairs each voxcore node with its ext entry by index, the placement the lossless
/// path emits. A vmax-origin scene is a tree with one ext node per voxcore node, so
/// this reproduces it exactly.
fn ext_placements<'a>(state: &'a VoxState, voxel_max: &VoxelMaxExt) -> Vec<Placement<'a>> {
    state
        .iter_hierarchy_nodes()
        .enumerate()
        .map(|(index, (_, node))| Placement {
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
/// placement along every path to it, so the rebuilt world is identical even though
/// Voxel Max models only a tree. Instances collapse back to one shared object when
/// the reverse path dedups them.
///
/// Lossy only where Voxel Max cannot represent the data from a bare scene: node
/// rotation is dropped to identity, since the reverse path stores an axis-angle the
/// voxcore quaternion is not inverted to, and the material palette name is left
/// empty. Node translation and scale, the hierarchy, colors, and any material
/// palette survive.
fn synthesize_placements(state: &VoxState) -> Vec<Placement<'_>> {
    let mut placements = Vec::new();
    let mut counter = 0usize;
    for &root in state.root_hierarchy_nodes() {
        push_placement(state, root, None, &mut counter, &mut placements);
    }
    placements
}

/// Emits a placement for the node `id` under `parent_id`, then recurses into its
/// child nodes. Each occurrence takes a fresh synthesized UUID, so a node reached
/// by several paths becomes a distinct scene node per path; child nodes attach to
/// this occurrence's id, which is also the id of the node's first object.
fn push_placement<'a>(
    state: &'a VoxState,
    id: U32Id<BVoxHierarchyNode>,
    parent_id: Option<String>,
    counter: &mut usize,
    placements: &mut Vec<Placement<'a>>,
) {
    let node = state.hierarchy_node(id).expect("a valid hierarchy node");
    let ext_id = synth_uuid(*counter);
    *counter += 1;
    // An object node carries its first object's grid as centered bounds; a group
    // node carries none, matching the bounds the reverse path re-bases against.
    let bounds = node.child_objects.first().and_then(|object| {
        state
            .object(*object)
            .map(|object| centered_bounds(object.bounds()))
    });
    let ext = VoxelMaxNode {
        id: ext_id.clone(),
        parent_id,
        index: None,
        rotation: Some(IDENTITY_AXIS_ANGLE),
        center: Some(bounds.map_or([0.0, 0.0, 0.0], |(center, _, _)| center)),
        bounds_min: bounds.map(|(_, min, _)| min),
        bounds_max: bounds.map(|(_, _, max)| max),
        alignment: Some(DEFAULT_ALIGNMENT.to_owned()),
        pivot_face: Some(DEFAULT_PIVOT_FACE.to_owned()),
        pivot_align: Some(DEFAULT_PIVOT_ALIGN.to_owned()),
        selected: None,
    };
    placements.push(Placement { node, ext });
    for &child in &node.child_nodes {
        push_placement(state, child, Some(ext_id.clone()), counter, placements);
    }
}

/// Synthesizes the scene-level `voxel-max` ext for a state that carries none, such
/// as one loaded from another format. It holds only the document-wide data with no
/// per-node home: a fallback scene version, a neutral camera, an empty name for each
/// palette, and no preserved object state. Per-node provenance comes from
/// [`synthesize_placements`].
fn synthesize_voxel_max_ext(state: &VoxState) -> VoxelMaxExt {
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

/// The per-object ext for an extra object on a node placing several, such as a Goxel
/// layer's blocks. It takes a distinct id and the object's own centered bounds, and
/// inherits the node's parent, rotation, and alignment to stay a sibling of the
/// node's first object.
fn secondary_object_ext(node_ext: &VoxelMaxNode, object: &VoxObject, slot: usize) -> VoxelMaxNode {
    let (center, bounds_min, bounds_max) = centered_bounds(object.bounds());
    VoxelMaxNode {
        id: secondary_uuid(&node_ext.id, slot),
        center: Some(center),
        bounds_min: Some(bounds_min),
        bounds_max: Some(bounds_max),
        ..node_ext.clone()
    }
}

/// The centered Voxel Max bounds `(e_c, e_mi, e_ma)` for an object grid of `size`:
/// `e_c` at the true box center with `e_mi`/`e_ma` as symmetric half-extents, the
/// convention Voxel Max reads. A node whose `e_c` is the box's min corner rather
/// than its center mislocates the object. The box still spans `[0, size]`, so the
/// re-based voxels at the grid origin are unchanged and `box_min` stays zero.
fn centered_bounds(size: TyVector3U32) -> ([f64; 3], [f64; 3], [f64; 3]) {
    let half = [
        size.x as f64 / 2.0,
        size.y as f64 / 2.0,
        size.z as f64 / 2.0,
    ];
    (half, [-half[0], -half[1], -half[2]], half)
}

/// A reference into one of an object's palettes: the sample reference id and the
/// palette id it names.
type PaletteRef = (U32Id<BVoxPaletteRef>, U32Id<BVoxPalette>);

/// The color and material palette references for an object, identified by the
/// `rgba` and `metallic` attributes.
fn object_palette_refs(
    state: &VoxState,
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
/// material indices from their sample references. `color_offset` is added to every
/// color index so a synthesized palette can reserve index 0 as the empty cell
/// Voxel Max expects; the lossless path passes zero.
///
/// Errors when a live voxel's shifted color index reaches [`PALETTE_COLORS`], the
/// reserved terminator. The check is per voxel rather than over the palette, so a
/// padded source palette such as MagicaVoxel's fixed 256 entries is fine as long as
/// its referenced colors fit; only a voxel that genuinely cannot be represented,
/// because the source uses more than the budget of colors, is rejected, rather than
/// wrapping its 8-bit index or colliding with the terminator and corrupting colors.
fn reconstruct_voxels(
    object: &VoxObject,
    color: Option<PaletteRef>,
    material: Option<PaletteRef>,
    ext_node: &VoxelMaxNode,
    color_offset: u8,
) -> Result<Vec<VMaxVoxel>> {
    let box_min = box_min(ext_node);
    object
        .iter_live()
        .map(|voxel| {
            let position = object
                .voxel_position(voxel)
                .expect("a live voxel is within the grid");
            let cell = |reference: Option<PaletteRef>| {
                reference.map_or(0, |(reference, _)| {
                    object
                        .voxel_cell(voxel, reference)
                        .expect("a live voxel samples every reference")
                        .to_u32()
                })
            };
            let color_idx = cell(color) + color_offset as u32;
            if color_idx >= PALETTE_COLORS as u32 {
                return Err(Error::invalid(format!(
                    "a voxel references color index {color_idx}, past the {} a Voxel Max \
                     palette holds, so the source has more colors than fit",
                    PALETTE_COLORS - 1
                )));
            }
            Ok(VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx: cell(material) as u8,
                color_idx: color_idx as u8,
            })
        })
        .collect()
}

/// Returns the `pal` filename for an object, building its color image and material
/// sidecar the first time the color palette is seen. An object with no color palette
/// borrows the default palette name and writes no file.
#[allow(clippy::too_many_arguments)]
fn build_palette(
    state: &VoxState,
    palette_names: &[Option<VoxelMaxPalette>],
    color: Option<U32Id<BVoxPalette>>,
    material: Option<U32Id<BVoxPalette>>,
    palette_files: &mut HashMap<usize, String>,
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    voxel_max_color_format: VoxelMaxColorFormat,
    color_offset: u8,
) -> String {
    let Some(color) = color else {
        // An object with no color palette borrows the default palette name; an empty
        // reference is one Voxel Max cannot resolve. No file is written for it.
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

    let colors = color_palette_colors(state, color, color_offset);
    if matches!(
        voxel_max_color_format,
        VoxelMaxColorFormat::Png | VoxelMaxColorFormat::All
    ) {
        let mut cells = colors.clone();
        cells.push([0, 0, 0, 0]);
        palette_png_files.insert(pal.clone(), VMaxPalettePngFile(cells));
    }
    // The settings sidecar carries the material, and the colors when no image does.
    // Plist mode writes no image, so even a color-only object with no material
    // writes its colors here rather than dropping them.
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

/// A color palette's cells as exactly [`PALETTE_COLORS`] RGBA entries, padded with
/// transparent cells or truncated so the count is fixed. `color_offset` transparent
/// cells lead the table, reserving the empty indices a synthesized palette shifts
/// its colors past; the lossless path passes zero. Cells past the budget are
/// dropped, which is harmless for a padded source palette whose extra entries no
/// voxel references; [`reconstruct_voxels`] rejects any voxel that would reference
/// one.
fn color_palette_colors(
    state: &VoxState,
    palette: U32Id<BVoxPalette>,
    color_offset: u8,
) -> Vec<[u8; 4]> {
    let color_offset = color_offset as usize;
    let palette = state.palette(palette).expect("a referenced palette");
    let rgba = palette
        .iter_attributes()
        .find(|(_, name)| *name == "rgba" || *name == "rgb")
        .map(|(id, _)| id);
    let mut cells: Vec<[u8; 4]> = vec![[0, 0, 0, 0]; color_offset];
    if let Some(rgba) = rgba {
        cells.extend(
            palette
                .iter_cells()
                .take(PALETTE_COLORS - color_offset)
                .map(|cell| parse_rgba(palette.cell_value(cell, rgba))),
        );
    }
    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    cells
}

/// Builds a settings sidecar carrying `colors` and, when `palette` names a material
/// palette, a material per cell read from its
/// `metallic`/`roughness`/`emissive`/`shadows` attributes and the optional
/// dispersion columns. A color-only palette passes `None` and gets no materials. The
/// editor-state keys are filled with the defaults Voxel Max expects, and each slot's
/// `mi` token from its 1-based position.
fn material_settings(
    state: &VoxState,
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

/// Parses a `#RRGGBB` or `#RRGGBBAA` color cell into RGBA bytes. A missing alpha,
/// as in the 6-hex form an `rgb` source carries, defaults to opaque.
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
        position: unbake_position(&node.transform, ext_node),
        rotation: ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE),
        scale: vector(node.transform.scale),
        ind,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone().unwrap_or_default(),
        t_pa: ext_node.pivot_align.clone().unwrap_or_default(),
        t_pf: ext_node.pivot_face.clone().unwrap_or_default(),
        t_po: None,
        center: ext_node.center.unwrap_or_default(),
        bounds_min: ext_node.bounds_min,
        bounds_max: ext_node.bounds_max,
    }
}

/// Builds a scene group from its node and preserved provenance.
fn group_from_node(node: &VoxHierarchyNode, ext_node: &VoxelMaxNode, ind: [i64; 3]) -> VMaxGroup {
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
        center: ext_node.center.unwrap_or_default(),
        bounds_min: ext_node.bounds_min,
        bounds_max: ext_node.bounds_max,
    }
}

/// The `ind` path triple for an emitted node: the preserved one from the ext, or a
/// synthesized triple keeping every node distinct, since Voxel Max collapses nodes
/// that share `[0, 0, 0]`. Groups take the `1` lane and objects the `0` lane.
fn node_ind(ext_node: &VoxelMaxNode, is_group: bool, counter: &mut i64) -> [i64; 3] {
    if let Some(index) = ext_node.index {
        return index;
    }
    let ind = [0, i64::from(is_group), *counter];
    *counter += 1;
    ind
}

/// The node's rotation as a quaternion, decoded from the stored axis-angle like the
/// read path so the two stay inverses. A degenerate axis decodes to identity.
fn ext_rotation(ext_node: &VoxelMaxNode) -> TyQuaternionF64 {
    let [x, y, z, angle] = ext_node.rotation.unwrap_or(IDENTITY_AXIS_ANGLE);
    TyQuaternionF64::from_axis_angle(TyVector3F64::new(x, y, z), angle)
}

/// The absolute model-space origin `round(center + bounds_min)` the forward path
/// re-based against.
fn box_min(ext_node: &VoxelMaxNode) -> [i32; 3] {
    let center = ext_node.center.unwrap_or_default();
    let min = ext_node.bounds_min.unwrap_or_default();
    [
        (center[0] + min[0]).round() as i32,
        (center[1] + min[1]).round() as i32,
        (center[2] + min[2]).round() as i32,
    ]
}

/// Recovers an object's `t_p`, the inverse of the read path's `object_transform`. It
/// backs out the `t_p` Voxel Max renders with from the node's transform and the
/// bounds center it pivots about. Uses the stored axis-angle rotation, not the live
/// transform's, so it stays an exact inverse when synthesis drops a node's rotation
/// to identity.
fn unbake_position(transform: &TyTransformF64, ext_node: &VoxelMaxNode) -> [f64; 3] {
    let center = ext_node.center.unwrap_or_default();
    let min = box_min(ext_node);
    let scale = transform.scale;
    let offset = TyVector3F64::new(
        (min[0] as f64 - center[0]) * scale.x,
        (min[1] as f64 - center[1]) * scale.y,
        (min[2] as f64 - center[2]) * scale.z,
    );
    let rotated = ext_rotation(ext_node).rotate(offset);
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

/// A syntactically valid, deterministic UUID for a synthesized scene node. Voxel Max
/// decodes a node's `id`/`pid` as a UUID and rejects a non-UUID token. The index is
/// offset by one so the first node avoids the all-zero nil UUID.
fn synth_uuid(index: usize) -> String {
    format!("00000000-0000-0000-0000-{:012X}", index + 1)
}

/// A distinct, valid UUID for an extra object on a node placing several, stamping
/// the object's slot into the node id's fourth group. A node id keeps that group
/// zero, so this never collides with a node or another slot.
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

#[cfg(test)]
mod tests {
    use crate::{
        VoxelMaxColorFormat, VoxelMaxExt, VoxelMaxExtWrapper, cell_color, from_vmax_file,
        object_color_ref, to_vmax_file, to_vox_value,
    };
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet};
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxObject, VMaxPalettePngFile,
        VMaxSceneCamera, VMaxSceneJsonFile,
    };
    use vmax_codec::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots};
    use voxcore::{
        BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, VoxHierarchyNode, VoxMap,
        VoxObject, VoxPalette, VoxState, VoxValue,
    };

    fn material(
        mi: &str,
        metalness: f64,
        roughness: f64,
        emission: f64,
        enable_shadows: bool,
    ) -> VMaxMaterial {
        VMaxMaterial {
            mi: mi.to_owned(),
            mc: metalness,
            rc: roughness,
            sic: emission,
            sh: enable_shadows,
            tc: None,
            md: None,
        }
    }

    /// The fixed editor-state defaults the reverse path writes on a rebuilt
    /// material palette, so a fixture round-trips to equality.
    fn palette_settings(
        name: &str,
        materials: Vec<VMaxMaterial>,
        colors: Vec<[u8; 4]>,
    ) -> vmax::VMaxPaletteSettingsVmaxpsbFile {
        vmax::VMaxPaletteSettingsVmaxpsbFile {
            name: name.to_owned(),
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

    /// A 256-cell image: 255 distinct colors plus the transparent terminator.
    fn palette_png() -> VMaxPalettePngFile {
        let mut cells: Vec<[u8; 4]> = (0..255u32).map(|i| [i as u8, 0, 0, 255]).collect();
        cells.push([0, 0, 0, 0]);
        VMaxPalettePngFile(cells)
    }

    /// A document with a root group, a child object placed at the origin with
    /// authored bounds, a shared color and material palette, and preserved scene
    /// and object editor state. Built in the canonical form the reverse path
    /// emits so it round-trips to equality.
    fn sample() -> VMaxFile {
        let group = VMaxGroup {
            name: "grp".to_owned(),
            id: "g".to_owned(),
            parent_id: None,
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [0.0, 0.0, 0.0],
            bounds_min: None,
            bounds_max: None,
        };
        let object = VMaxObject {
            name: "obj".to_owned(),
            data: "contents.vmaxb".to_owned(),
            palette: "palette.png".to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: "o".to_owned(),
            parent_id: Some("g".to_owned()),
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [0.0, 0.0, 0.0],
            bounds_min: Some([0.0, 0.0, 0.0]),
            bounds_max: Some([2.0, 2.0, 2.0]),
        };

        let scene_json_file = VMaxSceneJsonFile {
            v: 4,
            cam: Some(VMaxSceneCamera::default()),
            background: Some("#101010".to_owned()),
            groups: vec![group],
            objects: vec![object],
            ..Default::default()
        };

        // Canonical snapshots: voxels re-encoded just as the reverse path emits.
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[
                VMaxVoxel {
                    position: [0, 0, 0],
                    material_idx: 1,
                    color_idx: 5,
                },
                VMaxVoxel {
                    position: [1, 1, 1],
                    material_idx: 0,
                    color_idx: 3,
                },
            ]),
            uuid: "u".to_owned(),
            v: 4,
            tools: None,
            brush: None,
            cam: None,
            pal: None,
        };

        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents.vmaxb".to_owned(), contents);
        let mut palette_settings_files = BTreeMap::new();
        palette_settings_files.insert(
            "palette.settings.vmaxpsb".to_owned(),
            palette_settings(
                "mat",
                vec![
                    material("1", 0.0, 1.0, 0.0, true),
                    material("2", 0.5, 0.25, 2.0, false),
                ],
                Vec::new(),
            ),
        );
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert("palette.png".to_owned(), palette_png());

        VMaxFile {
            scene_json_file,
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
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let original = sample();
        let state = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(rebuilt, original);
    }

    /// A state carrying no format ext, built straight from voxcore: a red-green
    /// object and a blue object sharing one `rgba` palette, placed by a small
    /// hierarchy of a nested group and two roots. The writer must synthesize the
    /// `voxel-max` ext from this rather than read one. Red is cell 0, so a live
    /// voxel references the palette index Voxel Max reads as empty; synthesis must
    /// shift it past index 0 rather than drop the voxel.
    fn source_state() -> VoxState {
        let mut state = VoxState::default();

        // One rgba palette whose first real color is cell 0: red, green, blue.
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for hex in ["#FF0000FF", "#00FF00FF", "#0000FFFF"] {
            palette
                .add_cell(vec![VoxValue::Text(hex.to_owned())])
                .expect("one value per attribute");
        }
        let palette_id = state.add_palette(palette);
        let cell = |index: u32| U32Id::<BVoxPaletteCell>::from_u32(index);

        // Object 0: a red (cell 0) then a green (cell 1) voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.add_palette_ref(palette_id, cell(0));
        for (x, color) in [(0u32, 0u32), (1, 1)] {
            let voxel = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel, &[cell(color)])
                .expect("one sample for the one reference");
        }
        state.add_object(wide);

        // Object 1: a single blue (cell 2) voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.add_palette_ref(palette_id, cell(0));
        let voxel = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel, &[cell(2)])
            .expect("one sample for the one reference");
        state.add_object(unit);

        let object = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at = |x: f64, y: f64, z: f64| {
            TyTransformF64::new(
                TyVector3F64::new(x, y, z),
                TyQuaternionF64::identity(),
                TyVector3F64::new(1.0, 1.0, 1.0),
            )
        };

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "group".to_owned(),
            child_nodes: vec![node(1)],
            child_objects: Vec::new(),
            transform: TyTransformF64::default(),
        });
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "wide".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(0)],
            transform: placed_at(5.0, 0.0, 0.0),
        });
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "unit".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(1)],
            transform: placed_at(0.0, 3.0, 0.0),
        });
        state.set_root_hierarchy_nodes(vec![node(0), node(2)]);

        state.validate().expect("a well-formed source state");
        state
    }

    /// Every live voxel as `(world position, color)`, walking the hierarchy from
    /// the roots and accumulating each node's translation. Order-independent and
    /// resolved per voxel, so a state compares to one round-tripped through a
    /// synthesized document without depending on object, palette, or voxel order.
    fn world_voxels(state: &VoxState) -> BTreeSet<([i32; 3], [u8; 4])> {
        fn walk(
            state: &VoxState,
            node: U32Id<BVoxHierarchyNode>,
            origin: [i32; 3],
            voxels: &mut BTreeSet<([i32; 3], [u8; 4])>,
        ) {
            let node = state.hierarchy_node(node).expect("a valid node");
            let position = node.transform.position;
            let translation = [
                origin[0] + position.x.round() as i32,
                origin[1] + position.y.round() as i32,
                origin[2] + position.z.round() as i32,
            ];
            for &object in &node.child_objects {
                let object = state.object(object).expect("a valid object");
                let color = object_color_ref(state, object);
                for voxel in object.iter_live() {
                    let grid = object.voxel_position(voxel).expect("within the grid");
                    let world = [
                        translation[0] + grid.x as i32,
                        translation[1] + grid.y as i32,
                        translation[2] + grid.z as i32,
                    ];
                    let rgba = color.map_or([0, 0, 0, 0], |(reference, palette, attribute)| {
                        cell_color(object, voxel, reference, palette, attribute)
                    });
                    voxels.insert((world, rgba));
                }
            }
            for &child in &node.child_nodes {
                walk(state, child, translation, voxels);
            }
        }

        let mut voxels = BTreeSet::new();
        for &root in state.root_hierarchy_nodes() {
            walk(state, root, [0, 0, 0], &mut voxels);
        }
        voxels
    }

    /// A default state has no scene, so the writer synthesizes an empty document
    /// rather than erroring on the missing ext.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = VoxState::default();
        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert!(file.scene_json_file.objects.is_empty());
        assert!(file.scene_json_file.groups.is_empty());
    }

    /// A state with no `voxel-max` ext, such as one cross-loaded from another
    /// format, synthesizes a document that `from_vmax_file` reads back with the
    /// same world geometry, colors, and placement.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let source = source_state();
        let file = to_vmax_file(&source, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(world_voxels(&reloaded), world_voxels(&source));
    }

    /// A foreign ext, here `magica-voxel`, is ignored by the Voxel Max writer: it
    /// synthesizes from the scene rather than failing to parse the wrong ext.
    #[test]
    fn synthesizes_past_a_foreign_ext() {
        let mut source = source_state();
        source.set_ext(Some(VoxValue::Object(VoxMap(vec![(
            "magica-voxel".to_owned(),
            VoxValue::Null,
        )]))));
        let file = to_vmax_file(&source, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(world_voxels(&reloaded), world_voxels(&source));
    }

    /// Synthesis gives every node a distinct `ind`, since Voxel Max collapses nodes
    /// that share the `[0, 0, 0]` path triple onto one. Groups take the `1` lane and
    /// objects the `0` lane.
    #[test]
    fn synthesizes_distinct_node_ind() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // A root group parenting two object nodes.
        state.add_hierarchy_node(group_node("g", &[1, 2], at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("a", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("b", 1, at(5.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let scene = &file.scene_json_file;
        let inds: Vec<[i64; 3]> = scene
            .groups
            .iter()
            .map(|g| g.ind)
            .chain(scene.objects.iter().map(|o| o.ind))
            .collect();
        let distinct: BTreeSet<[i64; 3]> = inds.iter().copied().collect();
        assert_eq!(distinct.len(), inds.len(), "every node ind is distinct");
        assert!(scene.groups.iter().all(|g| g.ind[1] == 1));
        assert!(scene.objects.iter().all(|o| o.ind[1] == 0));
    }

    /// A node placing several objects, as a Goxel layer's blocks do, keeps every
    /// object: Voxel Max models one object per node, so synthesis flattens the
    /// extra objects to siblings that share the node's placement rather than
    /// dropping all but the first.
    #[test]
    fn synthesizes_a_node_placing_several_objects() {
        let mut state = VoxState::default();

        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for hex in ["#FF0000FF", "#00FF00FF"] {
            palette
                .add_cell(vec![VoxValue::Text(hex.to_owned())])
                .expect("one value per attribute");
        }
        let palette_id = state.add_palette(palette);
        let cell = |index: u32| U32Id::<BVoxPaletteCell>::from_u32(index);

        // Two unit objects, a red one (cell 0) and a green one (cell 1).
        for color in [0u32, 1] {
            let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
                .expect("a 1x1x1 grid is within the dense limit");
            object.add_palette_ref(palette_id, cell(0));
            let voxel = object
                .voxel_id(TyVector3U32::new(0, 0, 0))
                .expect("a position within the grid");
            object
                .retain_voxel(voxel, &[cell(color)])
                .expect("one sample for the one reference");
            state.add_object(object);
        }

        // One node placing both objects at the same offset, so they coincide in
        // the world at distinct colors.
        let object = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "layer".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(0), object(1)],
            transform: TyTransformF64::new(
                TyVector3F64::new(10.0, 0.0, 0.0),
                TyQuaternionF64::identity(),
                TyVector3F64::new(1.0, 1.0, 1.0),
            ),
        });
        state.set_root_hierarchy_nodes(vec![node(0)]);
        state.validate().expect("a well-formed source state");

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 2);
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([10, 0, 0], red), ([10, 0, 0], green)])
        );
    }

    /// An `rgba` palette holding the given colors.
    fn rgba_palette(hexes: &[&str]) -> VoxPalette {
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgba".to_owned());
        for hex in hexes {
            palette
                .add_cell(vec![VoxValue::Text((*hex).to_owned())])
                .expect("one value per attribute");
        }
        palette
    }

    /// An object of `bounds` whose live voxels each sample one color cell, given as
    /// `(position, cell)` pairs.
    fn color_object(
        palette: U32Id<BVoxPalette>,
        bounds: TyVector3U32,
        voxels: &[([u32; 3], u32)],
    ) -> VoxObject {
        let mut object = VoxObject::new(String::new(), bounds).expect("within the dense limit");
        object.add_palette_ref(palette, U32Id::<BVoxPaletteCell>::from_u32(0));
        for &([x, y, z], cell) in voxels {
            let id = object
                .voxel_id(TyVector3U32::new(x, y, z))
                .expect("a position within the grid");
            object
                .retain_voxel(id, &[U32Id::<BVoxPaletteCell>::from_u32(cell)])
                .expect("one sample for the one reference");
        }
        object
    }

    /// A translation-only transform.
    fn at(x: f64, y: f64, z: f64) -> TyTransformF64 {
        TyTransformF64::new(
            TyVector3F64::new(x, y, z),
            TyQuaternionF64::identity(),
            TyVector3F64::new(1.0, 1.0, 1.0),
        )
    }

    /// A node placing object `object` at `transform`.
    fn object_node(name: &str, object: u32, transform: TyTransformF64) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![U32Id::<BVoxObject>::from_u32(object)],
            transform,
        }
    }

    /// A group node parenting the given child nodes at `transform`.
    fn group_node(name: &str, children: &[u32], transform: TyTransformF64) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_nodes: children
                .iter()
                .map(|&c| U32Id::<BVoxHierarchyNode>::from_u32(c))
                .collect(),
            child_objects: Vec::new(),
            transform,
        }
    }

    /// The voxels decoded from a contents file's snapshots.
    fn contents_voxels(file: &VMaxFile, name: &str) -> Vec<VMaxVoxel> {
        decode_vmax_snapshots(&file.contents_files[name].snapshots).expect("decodable snapshots")
    }

    /// Synthesis reserves color index 0 as the empty cell Voxel Max expects: the
    /// image's first cell is transparent, the first real color sits at index 1, and
    /// every live voxel references a non-zero color, so a voxcore cell 0 is kept
    /// rather than read back as empty.
    #[test]
    fn synthesis_reserves_palette_index_zero() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let png = &file.palette_png_files["palette.png"].0;
        assert_eq!(png[0], [0, 0, 0, 0]);
        assert_eq!(png[1], [0xFF, 0, 0, 0xFF]);
        let voxels = contents_voxels(&file, "contents.vmaxb");
        assert!(!voxels.is_empty());
        assert!(voxels.iter().all(|voxel| voxel.color_idx >= 1));
    }

    /// The color offset never leaks into the lossless ext path: a state carrying a
    /// real `voxel-max` ext keeps a voxel that references color cell 0 at color
    /// index 0 in the emitted snapshots, the value Voxel Max wrote.
    #[test]
    fn lossless_path_keeps_color_index_zero() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        // A real, if minimal, voxel-max ext routes the write through the lossless
        // path, so the offset must stay zero.
        let ext = to_vox_value(&VoxelMaxExtWrapper {
            voxel_max: VoxelMaxExt {
                palettes: vec![None; state.palette_count()],
                ..Default::default()
            },
        })
        .unwrap();
        state.set_ext(Some(ext));
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let colors: Vec<u8> = file.contents_files["contents.vmaxb"]
            .snapshots
            .iter()
            .flat_map(|snapshot| snapshot.s.ds.chunks_exact(2).map(|pair| pair[1]))
            .collect();
        assert!(colors.iter().all(|&color| color == 0));
    }

    /// A palette with more colors than fit after reserving the empty index is
    /// rejected rather than silently truncated or wrapped: 254 colors is the
    /// synthesis budget and round-trips, 255 overflows it.
    #[test]
    fn errors_when_colors_exceed_palette_budget() {
        let synthesize = |count: u32| {
            let mut state = VoxState::default();
            let hexes: Vec<String> = (0..count).map(|i| format!("#{:06X}FF", i)).collect();
            let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
            let palette = state.add_palette(rgba_palette(&refs));
            let voxels: Vec<([u32; 3], u32)> = (0..count).map(|i| ([i, 0, 0], i)).collect();
            state.add_object(color_object(
                palette,
                TyVector3U32::new(count, 1, 1),
                &voxels,
            ));
            state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
            state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
            state.validate().unwrap();
            to_vmax_file(&state, VoxelMaxColorFormat::Png)
        };

        let file = synthesize(254).expect("254 colors fit the synthesis budget");
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            reloaded.object(U32Id::from_u32(0)).unwrap().live_count(),
            254
        );
        assert!(synthesize(255).is_err());
    }

    /// An object with no color palette borrows the default palette name rather than
    /// an empty `pal` Voxel Max cannot resolve, and writes no file of its own. The
    /// colored and empty objects share that name.
    #[test]
    fn synthesizes_a_colorless_object_sharing_the_default_palette() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        // Object 0: a single red voxel. Object 1: an empty, colorless object.
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap());
        state.add_hierarchy_node(object_node("colored", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("colorless", 1, at(10.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // Both objects name the one real palette; no extra or placeholder file.
        let names: BTreeSet<&str> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.palette.as_str())
            .collect();
        assert_eq!(names, BTreeSet::from(["palette.png"]));
        assert_eq!(
            file.palette_png_files.keys().collect::<Vec<_>>(),
            ["palette.png"]
        );

        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(world_voxels(&reloaded), BTreeSet::from([([0, 0, 0], red)]));
    }

    /// A node placing several objects and also parenting child nodes flattens to
    /// sibling object-nodes sharing the node's placement, with the child nodes
    /// hanging off the first object, and every object gets its own files.
    #[test]
    fn synthesizes_a_node_placing_objects_and_child_nodes() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF", "#0000FFFF"]));
        for cell in 0..3 {
            state.add_object(color_object(
                palette,
                TyVector3U32::new(1, 1, 1),
                &[([0, 0, 0], cell)],
            ));
        }
        // A fourth object placed by a child node, to confirm it hangs off the
        // first object of the multi-object parent.
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 0 places objects 0, 1, 2 at +10x and parents node 1; node 1 places
        // object 3 at +1y of the first object.
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "layer".to_owned(),
            child_nodes: vec![U32Id::<BVoxHierarchyNode>::from_u32(1)],
            child_objects: vec![
                U32Id::<BVoxObject>::from_u32(0),
                U32Id::<BVoxObject>::from_u32(1),
                U32Id::<BVoxObject>::from_u32(2),
            ],
            transform: at(10.0, 0.0, 0.0),
        });
        state.add_hierarchy_node(object_node("child", 3, at(0.0, 1.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // Four scene objects: three from the multi-object node, one from the child.
        assert_eq!(file.scene_json_file.objects.len(), 4);
        // The three siblings share the parent's id, and ids are all distinct.
        let ids: BTreeSet<&str> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect();
        assert_eq!(ids.len(), 4);

        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        let blue = [0, 0, 0xFF, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([
                ([10, 0, 0], red),
                ([10, 0, 0], green),
                ([10, 0, 0], blue),
                ([10, 1, 0], red),
            ])
        );
    }

    /// Two nodes placing one object instance it: the object's voxels are rebuilt
    /// once into a shared contents file, the scene carries two objects with
    /// distinct ids, and reloading collapses them back to one voxcore object placed
    /// at both positions.
    #[test]
    fn synthesizes_instances_sharing_one_contents_file() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("a", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("b", 0, at(20.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.contents_files.len(), 1);
        assert_eq!(file.scene_json_file.objects.len(), 2);
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 1);
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], red), ([20, 0, 0], red)])
        );
    }

    /// A subtree shared by several parents is placed once per parent: synthesis
    /// duplicates it per path so the rebuilt world matches voxcore, where a node's
    /// placement composes along every path to it.
    #[test]
    fn synthesizes_a_shared_subtree_at_every_parent() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 2 places object 0 at +1x and is a child of both group node 0 (at
        // the origin) and group node 1 (at +100x).
        state.add_hierarchy_node(group_node("a", &[2], at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(group_node("b", &[2], at(100.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("c", 0, at(1.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([1, 0, 0], red), ([101, 0, 0], red)])
        );
    }

    /// A node that is both a root and a child is placed at both, since voxcore
    /// renders it along each path; synthesis emits it once per path rather than
    /// dropping its root placement.
    #[test]
    fn synthesizes_a_node_that_is_root_and_child() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 1 places object 0 at +1x; it is both a root and a child of group
        // node 0 at +100x.
        state.add_hierarchy_node(group_node("a", &[1], at(100.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("c", 0, at(1.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([1, 0, 0], red), ([101, 0, 0], red)])
        );
    }

    /// A node reachable from no root is dropped, since voxcore never places it, so
    /// synthesis does not promote it to a spurious root.
    #[test]
    fn drops_a_node_unreachable_from_the_roots() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        // node 0 is a root placing object 0; node 1 places object 1 but is neither
        // a root nor anyone's child.
        state.add_hierarchy_node(object_node("rooted", 0, at(0.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("orphan", 1, at(50.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 1);
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(world_voxels(&reloaded), BTreeSet::from([([0, 0, 0], red)]));
    }

    /// Synthesis drops node rotation to identity but keeps translation and scale,
    /// the documented placement loss of a bare scene.
    #[test]
    fn drops_rotation_but_keeps_scale_and_translation() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        let transform = TyTransformF64::new(
            TyVector3F64::new(7.0, 0.0, 0.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), 90f64.to_radians()),
            TyVector3F64::new(2.0, 1.0, 1.0),
        );
        state.add_hierarchy_node(object_node("o", 0, transform));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let node = reloaded.hierarchy_node(U32Id::from_u32(0)).unwrap();
        assert_eq!(node.transform.rotation, TyQuaternionF64::identity());
        assert_eq!(node.transform.scale, TyVector3F64::new(2.0, 1.0, 1.0));
        assert_eq!(node.transform.position, TyVector3F64::new(7.0, 0.0, 0.0));
    }

    /// A material palette survives synthesis across every color format, and the
    /// material index is not shifted by the color offset.
    #[test]
    fn synthesizes_material_palettes_across_color_formats() {
        let formats = [
            VoxelMaxColorFormat::Png,
            VoxelMaxColorFormat::Plist,
            VoxelMaxColorFormat::All,
        ];
        for format in formats {
            let mut state = VoxState::default();
            let color = state.add_palette(rgba_palette(&["#FF0000FF"]));
            let mut materials = VoxPalette::default();
            for attribute in ["metallic", "roughness", "emissive", "shadows"] {
                materials.add_attribute(attribute.to_owned());
            }
            materials
                .add_cell(vec![
                    VoxValue::Number(0.5),
                    VoxValue::Number(0.25),
                    VoxValue::Number(2.0),
                    VoxValue::Bool(true),
                ])
                .unwrap();
            let material = state.add_palette(materials);
            let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
            object.add_palette_ref(color, U32Id::<BVoxPaletteCell>::from_u32(0));
            object.add_palette_ref(material, U32Id::<BVoxPaletteCell>::from_u32(0));
            let voxel = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
            object
                .retain_voxel(
                    voxel,
                    &[
                        U32Id::<BVoxPaletteCell>::from_u32(0),
                        U32Id::<BVoxPaletteCell>::from_u32(0),
                    ],
                )
                .unwrap();
            state.add_object(object);
            state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
            state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
            state.validate().unwrap();

            let file = to_vmax_file(&state, format).unwrap();
            // Materials are not offset: the one voxel keeps material index 0.
            assert!(
                contents_voxels(&file, "contents.vmaxb")
                    .iter()
                    .all(|voxel| voxel.material_idx == 0)
            );

            let reloaded = from_vmax_file(&file).unwrap();
            let material = reloaded
                .iter_palettes()
                .map(|(_, palette)| palette)
                .find(|palette| {
                    palette
                        .iter_attributes()
                        .any(|(_, name)| name == "metallic")
                })
                .expect("a material palette survives");
            let cell = material.iter_cells().next().expect("one material cell");
            let value = |name: &str| {
                let id = material
                    .iter_attributes()
                    .find(|(_, attribute)| *attribute == name)
                    .map(|(id, _)| id)
                    .unwrap();
                material.cell_value(cell, id).cloned()
            };
            assert_eq!(value("metallic"), Some(VoxValue::Number(0.5)));
            assert_eq!(value("roughness"), Some(VoxValue::Number(0.25)));
            assert_eq!(value("emissive"), Some(VoxValue::Number(2.0)));
            assert_eq!(value("shadows"), Some(VoxValue::Bool(true)));
        }
    }

    /// An `rgb` source palette carrying 6-hex colors widens to opaque RGBA: the
    /// missing alpha defaults to fully opaque rather than transparent.
    #[test]
    fn widens_rgb_source_to_opaque() {
        let mut state = VoxState::default();
        let mut palette = VoxPalette::default();
        palette.add_attribute("rgb".to_owned());
        palette
            .add_cell(vec![VoxValue::Text("#3366CC".to_owned())])
            .unwrap();
        let palette = state.add_palette(palette);
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(
            file.palette_png_files["palette.png"].0[1],
            [0x33, 0x66, 0xCC, 0xFF]
        );
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x33, 0x66, 0xCC, 0xFF])])
        );
    }

    /// Writing a colored single-object scene with no ext, reloading it, then
    /// writing it again reaches a fixed point: the second write takes the lossless
    /// path and reproduces the first document exactly.
    #[test]
    fn is_idempotent_for_a_colored_object() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(3.0, 4.0, 5.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file1 = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file1).unwrap();
        let file2 = to_vmax_file(&reloaded, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file2, file1);
    }

    /// A deep hierarchy of nested groups and object nodes round-trips: every leaf's
    /// world voxel survives the reconstructed parent chain.
    #[test]
    fn synthesizes_a_deep_hierarchy() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 1)],
        ));
        // root group 0 -> group 1 -> group 2 -> object node 3, plus object node 4
        // hanging off group 1, so leaves sit at different depths.
        state.add_hierarchy_node(group_node("r", &[1], at(1.0, 0.0, 0.0)));
        state.add_hierarchy_node(group_node("g1", &[2, 4], at(0.0, 2.0, 0.0)));
        state.add_hierarchy_node(group_node("g2", &[3], at(0.0, 0.0, 3.0)));
        state.add_hierarchy_node(object_node("leaf", 0, at(10.0, 0.0, 0.0)));
        state.add_hierarchy_node(object_node("mid", 1, at(0.0, 20.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([11, 2, 3], red), ([1, 22, 0], green)])
        );
    }

    /// An empty object synthesizes to a scene object with empty contents and keeps
    /// its bounds, rather than erroring or vanishing.
    #[test]
    fn synthesizes_an_empty_object_with_bounds() {
        let mut state = VoxState::default();
        let object = VoxObject::new(String::new(), TyVector3U32::new(3, 4, 5)).unwrap();
        state.add_object(object);
        state.add_hierarchy_node(object_node("empty", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 1);
        assert!(contents_voxels(&file, "contents.vmaxb").is_empty());
        // Centered bounds: e_c at the box center, e_ma a half-extent of the grid.
        assert_eq!(file.scene_json_file.objects[0].center, [1.5, 2.0, 2.5]);
        assert_eq!(
            file.scene_json_file.objects[0].bounds_max,
            Some([1.5, 2.0, 2.5])
        );
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            reloaded.object(U32Id::from_u32(0)).unwrap().bounds(),
            TyVector3U32::new(3, 4, 5)
        );
    }

    /// A fully transparent color round-trips as transparent rather than being
    /// dropped, distinct from the reserved empty index 0.
    #[test]
    fn round_trips_a_transparent_color() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#11223300"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        // Index 0 stays the reserved empty cell, the transparent color sits at 1.
        assert_eq!(file.palette_png_files["palette.png"].0[0], [0, 0, 0, 0]);
        assert_eq!(
            file.palette_png_files["palette.png"].0[1],
            [0x11, 0x22, 0x33, 0]
        );
        assert!(
            contents_voxels(&file, "contents.vmaxb")
                .iter()
                .all(|v| v.color_idx == 1)
        );
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x11, 0x22, 0x33, 0])])
        );
    }

    /// A padded source palette larger than the budget is fine as long as its
    /// referenced colors fit, matching a MagicaVoxel source whose fixed 256-entry
    /// palette uses only a few low indices. The size alone must not be rejected.
    #[test]
    fn synthesizes_a_padded_palette_using_low_indices() {
        let mut state = VoxState::default();
        let hexes: Vec<String> = (0..256).map(|i| format!("#{:06X}FF", i)).collect();
        let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let palette = state.add_palette(rgba_palette(&refs));
        // One voxel references a low cell; the other 255 cells are padding.
        state.add_object(color_object(
            palette,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 5)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Png).unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x00, 0x00, 0x05, 0xFF])])
        );
    }

    /// A color-only object keeps its colors in plist mode, where the colors ride in
    /// the settings sidecar rather than an image, instead of reloading as white.
    #[test]
    fn synthesizes_a_color_only_object_in_plist() {
        let mut state = VoxState::default();
        let palette = state.add_palette(rgba_palette(&["#FF0000FF", "#00FF00FF"]));
        state.add_object(color_object(
            palette,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ));
        state.add_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)));
        state.set_root_hierarchy_nodes(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)]);
        state.validate().unwrap();

        let file = to_vmax_file(&state, VoxelMaxColorFormat::Plist).unwrap();
        // No image in plist mode; the colors must still survive via the sidecar.
        assert!(file.palette_png_files.is_empty());
        assert!(!file.palette_settings_files.is_empty());
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], red), ([1, 0, 0], green)])
        );
    }
}
