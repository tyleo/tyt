use crate::{
    ABSORPTION, Error, ObjectPlacement, PALETTE_COLORS, Result, SHADOWS, SYNTH_CAMERA,
    SceneCameraSource, VMaxColorFormat, VMaxExtMaterial, VMaxExtNode, VMaxExtObjectState,
    VMaxExtPalette, VMaxVoxMain, VMaxWriteOptions, decode_axis_angle, encode_axis_angle,
    pbr_factor_to_vm_coefficient, place_object, tighten,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use ty_math::{TyBoundsF64, TyQuaternionF64, TyTransformF64, TyVector3F64};
use vmax::{
    VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxObject,
    VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneJsonFile,
    snapshots::{VMaxVoxel, encode_vmax_snapshots},
};
use voxcore::{
    BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty, VoxExt,
    VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValueColumn, VoxValuePool,
    color::ColorValues,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, ROUGHNESS, TRANSMISSION,
        default_scalar,
    },
};

/// The neutral default material Voxel Max fills unused slots with: matte, not
/// metallic, shadow-casting.
const DEFAULT_METALLIC: f64 = 0.1;

const DEFAULT_ROUGHNESS: f64 = 0.9;

/// The `pal` an object with no color palette borrows. An empty reference makes
/// Voxel Max read the package directory as a file and abort, so a colorless
/// object shares the first color palette's name and writes no file of its own.
const FALLBACK_PALETTE: &str = "palette1.png";

/// The material slots every Voxel Max palette carries. A color cell's material
/// is a bit in the settings `lc` byte, so at most 8 (0..=7) fit; the sidecar
/// always lists exactly this many, real materials in the low slots and the rest
/// padded with the neutral default.
const MATERIAL_SLOTS: usize = 8;

/// How far a node's rotation may drift from its preserved axis-angle before
/// the writer encodes the live rotation instead.
const ROTATION_TOLERANCE: f64 = 1e-9;

/// Writes a [`VMaxVoxMain`] to a Voxel Max document, the inverse of
/// [`from_vmax_file`](crate::from_vmax_file()). A loaded document writes back
/// exactly through its ext. A state
/// [`to_vmax_vox_main`](crate::to_vmax_vox_main()) gave its ext writes as a
/// document synthesized from the scene. The ext supplies each node's,
/// palette's, and object's provenance and the scene-level state. The scene
/// supplies the rest: names, transforms, parents, and content boxes. Nodes
/// write in listing order, children before parents when the listing has them
/// so, as Voxel Max's documents do. Every object and node transform turns back
/// onto Voxel Max's Z-up axes. Each object's one palette is read unconverted in
/// Voxel Max's layout, the one the loader builds. Errors when an entity has no
/// ext entry, when an object has other than one layer, or when a palette
/// departs from the layout.
pub fn to_vmax_file(main: &VMaxVoxMain, options: &VMaxWriteOptions) -> Result<VMaxFile> {
    let placements = ext_placements(main)?;

    let mut objects: Vec<VMaxObject> = Vec::new();
    let mut groups: Vec<VMaxGroup> = Vec::new();
    // One plan per palette in first-seen order, plus one shared by every
    // layerless object, and each plan's position.
    let mut plans: Vec<PalettePlan> = Vec::new();
    let mut plan_index_of: HashMap<Option<U32Id<BVoxPalette>>, usize> = HashMap::new();
    // Object id -> its `data` filename, shared by every node that places it.
    let mut contents_by_object: BTreeMap<U32Id<BVoxObject>, String> = BTreeMap::new();

    let mut contents_files: BTreeMap<String, VMaxContentsVmaxbFile> = BTreeMap::new();
    let mut palette_settings_files: BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile> =
        BTreeMap::new();
    let mut palette_png_files: BTreeMap<String, VMaxPalettePngFile> = BTreeMap::new();

    // Every node's `ind` is distinct through its entry. An extra object on a
    // node placing several is emitted without an entry, so it takes a triplet
    // no entry uses.
    let mut used_indices: HashSet<[i64; 3]> = placements
        .iter()
        .map(|placement| placement.ext.index)
        .collect();

    // A group's content box is derived from its subtree, the same box for every
    // path to a shared node, so it is memoized by node id.
    let mut box_memo: HashMap<u32, ([f64; 3], [f64; 3])> = HashMap::new();

    for placement in &placements {
        let node = placement.node;
        let ext_node = placement.ext;
        let transform = node.transform.yup_to_zup();
        let rotation = node_rotation(ext_node, &transform);

        if node.child_object_ids.is_empty() {
            let (center, half) = subtree_box_local(main, placement.node_id, &mut box_memo);
            groups.push(group_from_node(
                placement, &transform, rotation, center, half,
            ));
            continue;
        }

        // One scene object per child object. A vmax-origin node always carries
        // a single object, so this loops once and matches the lossless path
        // exactly. A synthesized node may carry several, such as a Goxel
        // layer's blocks; the extra objects become sibling object-nodes under
        // the same parent.
        for (slot, object_id) in node.child_object_ids.iter().enumerate() {
            let object_id = *object_id;
            let object = main.object(object_id).expect("a valid node child object");
            let palette_id = object_layer(object)?.map(|(_, palette_id)| palette_id);
            let plan_index = match plan_index_of.get(&palette_id) {
                Some(&plan_index) => plan_index,

                None => {
                    let colored_count = plans.iter().filter(|p| p.color_table.is_some()).count();
                    plans.push(match palette_id {
                        Some(palette_id) => new_palette_plan(main, palette_id, colored_count)?,
                        None => colorless_plan(),
                    });
                    plan_index_of.insert(palette_id, plans.len() - 1);
                    plans.len() - 1
                }
            };
            let plan = &plans[plan_index];
            let suffix = object_file_suffix(object_id);
            // The node's ext places its first object. An extra object gets a
            // per-object variant with a distinct id and index and its own
            // bounds.
            let object_ext = if slot == 0 {
                ext_node.clone()
            } else {
                secondary_object_ext(ext_node, slot, &mut used_indices)
            };

            // Re-derive the object's internal-grid placement by convention:
            // center the canvas in the 256-wide workspace, then seat the
            // runtime grid inside it by the runtime/edit origin offset. The
            // runtime grid is the live voxels' tight extent within the object's
            // build volume; the content box follows from it and the build
            // volume, the scene placement from the node transform. The object
            // turns back onto Z-up axes with its transform.
            let (tight, object_placement) = place_object(&object.yup_to_zup());
            let object_state = ext_entry(
                main.ext().object_states.get(&object_id),
                "object",
                object_id.to_u32(),
            )?;

            // Instances share one contents file: rebuild it once.
            let data = match contents_by_object.get(&object_id) {
                Some(data) => data.clone(),

                None => {
                    let voxels = reconstruct_voxels(&tight, plan, object_placement.box_min)?;
                    let data = format!("contents{suffix}.vmaxb");
                    // Voxels re-encode into snapshots; serde-only state the
                    // decoded voxcore object does not model (`pal`) stays
                    // absent.
                    let contents = VMaxContentsVmaxbFile {
                        snapshots: encode_vmax_snapshots(&voxels),
                        ..contents_editor_state(object_state, &object_placement)
                    };
                    contents_files.insert(data.clone(), contents);
                    contents_by_object.insert(object_id, data.clone());
                    data
                }
            };

            let pal = plan.pal.clone();

            objects.push(object_from_node(
                &node.name,
                &transform,
                &object_ext,
                placement.parent_id.clone(),
                rotation,
                &object_placement,
                data,
                pal,
                &suffix,
            ));
        }
    }

    write_palette_files(
        &plans,
        &mut palette_settings_files,
        &mut palette_png_files,
        options.color_format,
    )?;

    let mut scene = main.ext().scene.clone();
    scene.groups = groups;
    scene.objects = objects;
    apply_scene_camera(&mut scene, options.scene_camera);

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
/// voxcore node supplies the local transform, the ext supplies the id and
/// anchors, and the scene supplies the parent.
struct Placement<'a> {
    node_id: U32Id<BVoxHierarchyNode>,

    node: &'a VoxHierarchyNode,

    ext: &'a VMaxExtNode,

    parent_id: Option<String>,
}

/// Pairs each voxcore node, in listing order, with its ext entry by id and
/// its parent's id from the scene. Voxel Max holds a tree. Errors when:
///
/// 1. a node has no entry, which means the ext is out of step
/// 2. a node has more than one parent
/// 3. a root is also a child, because a written node with a parent is no
///    longer a root
fn ext_placements(main: &VMaxVoxMain) -> Result<Vec<Placement<'_>>> {
    let ext = main.ext();

    let mut parent_ids: BTreeMap<U32Id<BVoxHierarchyNode>, U32Id<BVoxHierarchyNode>> =
        BTreeMap::new();
    for (parent_id, node) in main.iter_hierarchy_nodes() {
        for &child_id in &node.child_node_ids {
            if let Some(&other_id) = parent_ids.get(&child_id) {
                return Err(Error::invalid(format!(
                    "node {} has parents {} and {}, but a Voxel Max node has one parent",
                    child_id.to_u32(),
                    other_id.to_u32(),
                    parent_id.to_u32()
                )));
            }
            parent_ids.insert(child_id, parent_id);
        }
    }

    for &root_id in main.root_hierarchy_node_ids() {
        if let Some(parent_id) = parent_ids.get(&root_id) {
            return Err(Error::invalid(format!(
                "node {} is a root and a child of node {}, but a Voxel Max node with a parent \
                 is not a root",
                root_id.to_u32(),
                parent_id.to_u32()
            )));
        }
    }

    main.iter_hierarchy_nodes()
        .map(|(node_id, node)| {
            let entry = |id: U32Id<BVoxHierarchyNode>| {
                ext.hierarchy_nodes.get(&id).ok_or_else(|| {
                    Error::invalid(format!("vmax ext holds no entry for node {}", id.to_u32()))
                })
            };
            let parent_id = match parent_ids.get(&node_id) {
                Some(&parent_id) => Some(entry(parent_id)?.id.clone()),
                None => None,
            };
            Ok(Placement {
                node_id,
                node,
                ext: entry(node_id)?,
                parent_id,
            })
        })
        .collect()
}

/// The axis-angle a node writes: the preserved spelling while it still
/// decodes to the node's rotation, which keeps a loaded document byte for
/// byte, or the live rotation encoded afresh once the node was rotated after
/// the load. `transform` places the node on Voxel Max's Z-up axes.
fn node_rotation(ext_node: &VMaxExtNode, transform: &TyTransformF64) -> [f64; 4] {
    let stored = decode_axis_angle(ext_node.rotation);
    let live = transform.rotation;
    if stored.abs_diff_eq(live, ROTATION_TOLERANCE) || stored.abs_diff_eq(-live, ROTATION_TOLERANCE)
    {
        return ext_node.rotation;
    }
    encode_axis_angle(live)
}

/// The bounding box `(center, half)` of all geometry under `node_id`, in that
/// node's local frame on Voxel Max's Z-up axes: the union of each child
/// object's content box and each child node's box mapped through the child's
/// transform on those axes. Voxel Max stores this per group as
/// `e_c`/`e_mi`/`e_ma`; it is the union of the subtree, so it is derived here
/// rather than kept in the ext. Memoized by node id so a subtree shared across
/// parents is walked once. A node with no geometry collapses to a zero box.
fn subtree_box_local<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: U32Id<BVoxHierarchyNode>,
    memo: &mut HashMap<u32, ([f64; 3], [f64; 3])>,
) -> ([f64; 3], [f64; 3]) {
    if let Some(&box_local) = memo.get(&node_id.to_u32()) {
        return box_local;
    }
    let node = main
        .hierarchy_node(node_id)
        .expect("a valid hierarchy node");
    let mut bounds: Option<([f64; 3], [f64; 3])> = None;
    for &object_id in &node.child_object_ids {
        let (center, half) = object_box_local(main, object_id);
        extend_bounds(&mut bounds, center, half);
    }
    for &child_id in &node.child_node_ids {
        let (child_center, child_half) = subtree_box_local(main, child_id, memo);
        let transform = main
            .hierarchy_node(child_id)
            .expect("a valid child node")
            .transform
            .yup_to_zup();
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
/// frame on Voxel Max's Z-up axes: the tight runtime grid `[origin, origin +
/// bounds]`. An empty object has no runtime extent, so it frames its build
/// volume instead, matching the content box the write path gives it.
fn object_box_local<T: VoxExt>(
    main: &VoxMain<T>,
    object_id: U32Id<BVoxObject>,
) -> ([f64; 3], [f64; 3]) {
    let object = main.object(object_id).expect("a valid child object");
    let (tight, (edit_bounds, edit_origin)) = tighten(&object.yup_to_zup());
    let bounds = tight.bounds();
    let box_local = if bounds.x == 0 && bounds.y == 0 && bounds.z == 0 {
        TyBoundsF64::from_min_size(edit_origin.as_dvec3(), edit_bounds.as_dvec3())
    } else {
        TyBoundsF64::from_min_size(tight.origin().as_dvec3(), bounds.as_dvec3())
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
            *min = TyVector3F64::from_array(*min).min(lo).to_array();
            *max = TyVector3F64::from_array(*max).max(hi).to_array();
        }

        None => *bounds = Some((lo.to_array(), hi.to_array())),
    }
}

/// The half-extent of the AABB of a box rotated and scaled by a node transform.
/// A box centered on its pivot stays centered under the transform, so only the
/// half-extent picks up the rotation: `sum_j abs(col_j) * half[j]` over the
/// rotated, scaled basis columns.
fn transform_half(transform: &TyTransformF64, half: [f64; 3]) -> [f64; 3] {
    let col_x = transform.rotation * TyVector3F64::new(transform.scale.x, 0.0, 0.0);
    let col_y = transform.rotation * TyVector3F64::new(0.0, transform.scale.y, 0.0);
    let col_z = transform.rotation * TyVector3F64::new(0.0, 0.0, transform.scale.z);
    [
        col_x.x.abs() * half[0] + col_y.x.abs() * half[1] + col_z.x.abs() * half[2],
        col_x.y.abs() * half[0] + col_y.y.abs() * half[1] + col_z.y.abs() * half[2],
        col_x.z.abs() * half[0] + col_y.z.abs() * half[1] + col_z.z.abs() * half[2],
    ]
}

/// The content box `(center, half)` is the group's derived subtree box,
/// written as the symmetric `e_c`, `e_mi`, and `e_ma` Voxel Max stores.
/// `transform` places the node on Voxel Max's Z-up axes.
fn group_from_node(
    placement: &Placement<'_>,
    transform: &TyTransformF64,
    rotation: [f64; 4],
    center: [f64; 3],
    half: [f64; 3],
) -> VMaxGroup {
    let node = placement.node;
    let ext_node = placement.ext;
    VMaxGroup {
        name: node.name.clone(),
        id: ext_node.id.clone(),
        parent_id: placement.parent_id.clone(),
        hidden: None,
        position: transform.position.to_array(),
        rotation,
        scale: transform.scale.to_array(),
        ind: ext_node.index,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone(),
        t_pa: ext_node.pivot_align.clone(),
        t_pf: ext_node.pivot_face.clone(),
        t_po: None,
        center,
        bounds_min: Some([-half[0], -half[1], -half[2]]),
        bounds_max: Some(half),
    }
}

/// The one layer `object` samples and the palette it draws, or `None` for an
/// object with no layer, which writes colorless. A Voxel Max object reads one
/// palette and this writer converts nothing, so a second layer errors.
fn object_layer(object: &VoxObject) -> Result<Option<(U32Id<BVoxLayer>, U32Id<BVoxPalette>)>> {
    let mut layers = object.iter_layers();
    match (layers.next(), layers.next()) {
        (layer, None) => Ok(layer),

        _ => Err(Error::invalid(format!(
            "object \"{}\" has {} layers but a Voxel Max object reads one palette",
            object.name(),
            object.layer_count()
        ))),
    }
}

/// The Voxel Max palette one voxcore palette writes, shared by every object
/// layering it.
struct PalettePlan {
    /// The `pal` filename the objects reference.
    pal: String,

    /// The display name for the settings sidecar.
    name: String,

    /// The color table, the `baseColor` value pool whole. `None` for a
    /// colorless palette, which borrows a name and writes no file.
    color_table: Option<Vec<[u8; 4]>>,

    /// The indices each material writes.
    indices: BTreeMap<U32Id<BVoxMaterial>, VoxelIndices>,

    /// The materials in slot order. Empty when the palette binds no material
    /// axis, which leaves Voxel Max its own defaults.
    materials: Vec<VMaxMaterial>,
}

/// Plans the Voxel Max palette for `palette_id`. Voxel Max numbers palette
/// files 1-based (`palette1`, `palette2`, ...), by `colored_count`, since an
/// un-numbered `palette.png` breaks the plist color lookup when no image is
/// written. Errors when the palette has no ext entry or departs from the
/// [`PaletteLayout`]. A palette with materials but no `baseColor` errors
/// because Voxel Max keeps a material list only in a color palette's sidecar.
/// An `emissiveColor` with no material axis errors because its default
/// strength glows where Voxel Max's default materials do not.
fn new_palette_plan(
    main: &VMaxVoxMain,
    palette_id: U32Id<BVoxPalette>,
    colored_count: usize,
) -> Result<PalettePlan> {
    let provenance = ext_entry(
        main.ext().palettes.get(&palette_id),
        "palette",
        palette_id.to_u32(),
    )?;
    let layout = PaletteLayout::resolve(main, palette_id)?;

    let color_table = match &layout.color {
        Some(color) => Some(color_palette_colors(color)?),
        None => None,
    };
    if color_table.is_none() && !layout.material.is_empty() {
        return Err(Error::invalid(format!(
            "palette {} binds materials but no `{BASE_COLOR}`, and Voxel Max keeps a material \
             list only in a color palette's sidecar",
            palette_id.to_u32()
        )));
    }
    if layout.material.is_empty() && layout.emissive_color.is_some() {
        return Err(Error::invalid(format!(
            "palette {} binds `{EMISSIVE_COLOR}` but no material property to carry its \
             strength, which Voxel Max's default materials would not glow at",
            palette_id.to_u32()
        )));
    }
    // An empty reference is one Voxel Max cannot resolve, so a colorless
    // palette borrows the default name and writes no file.
    let pal = match color_table {
        Some(_) => format!("palette{}.png", colored_count + 1),
        None => FALLBACK_PALETTE.to_owned(),
    };

    let materials = slot_materials(&layout, provenance)?;
    let mut indices = BTreeMap::new();
    for material_id in layout.palette.iter_materials() {
        let entry = voxel_indices(&layout, material_id)?;
        if let Some(material) = materials.get(usize::from(entry.material_idx)) {
            check_emissive(&layout, material_id, material.sic)?;
        }
        indices.insert(material_id, entry);
    }

    Ok(PalettePlan {
        pal,
        name: provenance.name.clone(),
        color_table,
        indices,
        materials,
    })
}

/// A palette read in Voxel Max's layout, the one
/// [`from_vmax_file`](crate::from_vmax_file()) builds. The color axis is
/// `baseColor` and `emissiveColor`, one value per color cell. The material axis
/// is every other property, one value per material slot. A material is one cell
/// with one slot, so its color-axis properties share a value id and its
/// material-axis properties share another. The writer converts nothing: a
/// palette off the layout errors where it departs.
struct PaletteLayout<'a> {
    palette: &'a VoxPalette,

    /// `baseColor`.
    color: Option<ColorProperty<'a>>,

    /// `emissiveColor`.
    emissive_color: Option<ColorProperty<'a>>,

    /// The material axis, in property order.
    material: Vec<LayoutProperty<'a>>,
}

impl<'a> PaletteLayout<'a> {
    /// Reads palette `palette_id` of `main`. Errors when the state does not
    /// hold it.
    fn resolve<T: VoxExt>(main: &'a VoxMain<T>, palette_id: U32Id<BVoxPalette>) -> Result<Self> {
        let palette = main.palette(palette_id).ok_or_else(|| {
            Error::invalid(format!(
                "an object layers palette {}, which the state does not hold",
                palette_id.to_u32()
            ))
        })?;
        let mut color = None;
        let mut emissive_color = None;
        let mut material = Vec::new();
        for (id, property) in palette.iter_properties() {
            let value_pool = main
                .value_pool(property.value_pool_id)
                .expect("a property draws from a live value pool");

            let name = property.name.as_str();
            match name {
                BASE_COLOR => color = Some(ColorProperty::resolve(id, name, value_pool)?),

                EMISSIVE_COLOR => {
                    emissive_color = Some(ColorProperty::resolve(id, name, value_pool)?)
                }

                _ => material.push(LayoutProperty {
                    id,
                    name,
                    value_pool,
                }),
            }
        }

        Ok(PaletteLayout {
            palette,
            color,
            emissive_color,
            material,
        })
    }

    /// The material-axis property named `name`, or `None` when the palette
    /// does not bind it.
    fn material_property(&self, name: &str) -> Option<&LayoutProperty<'a>> {
        self.material.iter().find(|property| property.name == name)
    }
}

/// A palette property with the value pool it draws from.
struct LayoutProperty<'a> {
    id: U32Id<BVoxProperty>,

    name: &'a str,

    value_pool: &'a VoxValuePool,
}

/// A color-axis property with its value pool read as colors.
struct ColorProperty<'a> {
    id: U32Id<BVoxProperty>,

    value_pool: &'a VoxValuePool,

    colors: ColorValues,
}

impl<'a> ColorProperty<'a> {
    /// Reads property `id` over `value_pool`. Errors with `name` when the value
    /// pool holds no colors.
    fn resolve(id: U32Id<BVoxProperty>, name: &str, value_pool: &'a VoxValuePool) -> Result<Self> {
        let colors = ColorValues::of(value_pool).ok_or_else(|| {
            Error::invalid(format!("`{name}` draws from a value pool holding no color"))
        })?;

        Ok(Self {
            id,
            value_pool,
            colors,
        })
    }
}

/// `color`'s value pool decoded to exactly [`PALETTE_COLORS`] 0-based RGBA
/// entries, padded with transparent entries to that count.
///
/// Errors when the value pool holds more colors than the budget, because the
/// table is the pool whole.
fn color_palette_colors(color: &ColorProperty) -> Result<Vec<[u8; 4]>> {
    if color.value_pool.len() > PALETTE_COLORS {
        return Err(Error::invalid(format!(
            "`{BASE_COLOR}` draws from a value pool of {} colors, but a Voxel Max palette holds \
             only {PALETTE_COLORS}",
            color.value_pool.len()
        )));
    }

    let mut cells: Vec<[u8; 4]> = color
        .value_pool
        .iter_value_ids()
        .map(|value_id| {
            color
                .colors
                .srgba_u8(value_id)
                .expect("a listed value id is the value pool's")
        })
        .collect();

    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    Ok(cells)
}

/// The material list a palette writes, in slot order: an exact list in
/// `provenance` as it is, else one material per slot read from the
/// material-axis pools. Empty when the palette binds no material axis,
/// leaving Voxel Max its own defaults.
///
/// Every material-axis pool holds one value per slot, so the pools must be
/// the same length, densely numbered from zero, and no longer than
/// [`MATERIAL_SLOTS`]. An exact list must be that length too. Anything else
/// errors, including a pruned pool not yet compacted.
fn slot_materials(
    layout: &PaletteLayout,
    provenance: &VMaxExtPalette,
) -> Result<Vec<VMaxMaterial>> {
    let Some(first) = layout.material.first() else {
        if !provenance.materials.is_empty() {
            return Err(Error::invalid(format!(
                "vmax ext lists {} exact materials, but the palette binds no material property \
                 to index them",
                provenance.materials.len()
            )));
        }
        return Ok(Vec::new());
    };
    let slot_count = first.value_pool.len();
    for property in &layout.material {
        let dense = property.value_pool.len() == slot_count
            && (0..slot_count).all(|slot| {
                property
                    .value_pool
                    .contains_value(U32Id::from_u32(slot as u32))
            });
        if !dense {
            return Err(Error::invalid(format!(
                "`{}` holds {} values where `{}` holds {slot_count}, but a Voxel Max material \
                 pool holds one value per slot, numbered from zero",
                property.name,
                property.value_pool.len(),
                first.name
            )));
        }
    }
    if slot_count > MATERIAL_SLOTS {
        return Err(Error::invalid(format!(
            "the material pools hold {slot_count} values, but a Voxel Max palette holds only \
             {MATERIAL_SLOTS} material slots"
        )));
    }
    if !provenance.materials.is_empty() {
        if provenance.materials.len() != slot_count {
            return Err(Error::invalid(format!(
                "vmax ext lists {} exact materials, but the material pools hold {slot_count} \
                 values",
                provenance.materials.len()
            )));
        }
        return Ok(provenance
            .materials
            .iter()
            .enumerate()
            .map(|(slot, material)| vmax_material(slot, material))
            .collect());
    }
    let pools = MaterialPools::resolve(layout)?;

    (0..slot_count)
        .map(|slot| pool_material(&pools, slot as u8))
        .collect()
}

/// The material-axis pools [`pool_material`] reads, each typed once. `None`
/// where the palette does not bind the property.
struct MaterialPools<'a> {
    metallic: Option<VoxValueColumn<'a, f64>>,

    roughness: Option<VoxValueColumn<'a, f64>>,

    emissive_strength: Option<VoxValueColumn<'a, f64>>,

    shadows: Option<VoxValueColumn<'a, bool>>,

    absorption: Option<VoxValueColumn<'a, f64>>,

    ior: Option<VoxValueColumn<'a, f64>>,

    transmission: Option<VoxValueColumn<'a, f64>>,
}

impl<'a> MaterialPools<'a> {
    /// Reads `layout`'s material axis. Errors when a bound scalar's pool
    /// holds no floats or `shadows`'s pool holds no flags.
    fn resolve(layout: &PaletteLayout<'a>) -> Result<Self> {
        let scalar = |name: &str| -> Result<Option<VoxValueColumn<'a, f64>>> {
            let Some(property) = layout.material_property(name) else {
                return Ok(None);
            };

            property.value_pool.float_values().map(Some).ok_or_else(|| {
                Error::invalid(format!(
                    "`{name}` draws from a value pool holding no scalar"
                ))
            })
        };

        let shadows = match layout.material_property(SHADOWS) {
            None => None,

            Some(property) => Some(property.value_pool.boolean_values().ok_or_else(|| {
                Error::invalid(format!(
                    "`{SHADOWS}` draws from a value pool holding no flag"
                ))
            })?),
        };

        Ok(Self {
            metallic: scalar(METALLIC)?,
            roughness: scalar(ROUGHNESS)?,
            emissive_strength: scalar(EMISSIVE_STRENGTH)?,
            shadows,
            absorption: scalar(ABSORPTION)?,
            ior: scalar(IOR)?,
            transmission: scalar(TRANSMISSION)?,
        })
    }
}

/// Rebuilds a Voxel Max material from its exact ext copy. The `mi` token is
/// derived from the 1-based slot and the transparency color `tc` is dropped,
/// matching the writer's behavior.
fn vmax_material(slot: usize, material: &VMaxExtMaterial) -> VMaxMaterial {
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

/// The Voxel Max material in slot `slot`, read from each material-axis pool at
/// the slot's value id. Metalness and roughness map from the 0 to 1 factor to
/// Voxel Max's 0.1 to 0.9 slider coefficient; see
/// [`pbr_factor_to_vm_coefficient`]. A property the palette does not bind
/// writes its vocabulary default, so it writes what it renders as.
fn pool_material(pools: &MaterialPools, slot: u8) -> Result<VMaxMaterial> {
    let value_id = U32Id::from_u32(u32::from(slot));

    // `slot_materials` checked that every material-axis pool holds each slot.
    let read = |values: Option<VoxValueColumn<'_, f64>>| -> Option<f64> {
        values.map(|values| *values.get(value_id).expect("a dense pool holds every slot"))
    };

    let dispersed =
        pools.ior.is_some() || pools.transmission.is_some() || pools.absorption.is_some();

    Ok(VMaxMaterial {
        mi: (usize::from(slot) + 1).to_string(),
        mc: pbr_factor_to_vm_coefficient(unbound_scalar(read(pools.metallic), METALLIC), METALLIC)?,
        rc: pbr_factor_to_vm_coefficient(
            unbound_scalar(read(pools.roughness), ROUGHNESS),
            ROUGHNESS,
        )?,
        // An unbound strength glows nowhere: the loader binds one whenever a
        // material glows.
        sic: read(pools.emissive_strength).unwrap_or(0.0),
        // Voxel Max casts shadows by default.
        sh: pools
            .shadows
            .map(|shadows| {
                *shadows
                    .get(value_id)
                    .expect("a dense pool holds every slot")
            })
            .unwrap_or(true),
        tc: None,
        md: match dispersed {
            true => Some(VMaxMaterialDispersion {
                absorption: read(pools.absorption).unwrap_or(0.0),
                ior: unbound_scalar(read(pools.ior), IOR),
                transmission: unbound_scalar(read(pools.transmission), TRANSMISSION),
            }),

            false => None,
        },
    })
}

/// `value` when the property is bound, else the glTF vocabulary default the
/// format gives `key`, so an unbound property writes what it renders as.
fn unbound_scalar(value: Option<f64>, key: &str) -> f64 {
    value.unwrap_or_else(|| {
        default_scalar(key).expect("a vocabulary scalar the vmax writer emits has a spec default")
    })
}

/// The Voxel Max indices a voxel writes.
#[derive(Clone, Copy)]
struct VoxelIndices {
    /// The 1-based color cell.
    color_idx: u8,

    /// The 0-based material slot.
    material_idx: u8,
}

/// The indices a voxel drawing material `material_id` writes.
fn voxel_indices(layout: &PaletteLayout, material_id: U32Id<BVoxMaterial>) -> Result<VoxelIndices> {
    Ok(VoxelIndices {
        color_idx: color_cell(layout, material_id)?,
        material_idx: material_slot(layout, material_id)?,
    })
}

/// The 1-based color cell material `material_id` draws: its `baseColor` value
/// id plus one, since the color table is that value pool whole and cell 0 is
/// the empty cell. One when the palette binds no color. Errors when the cell
/// reaches [`PALETTE_COLORS`].
fn color_cell(layout: &PaletteLayout, material_id: U32Id<BVoxMaterial>) -> Result<u8> {
    let Some(color) = &layout.color else {
        return Ok(1);
    };
    let cell = layout
        .palette
        .value_id(material_id, color.id)
        .expect("a live material has a value id for every property")
        .to_u32();
    if cell >= PALETTE_COLORS as u32 {
        return Err(Error::invalid(format!(
            "material {} draws color cell {cell}, but a Voxel Max palette holds only \
             {PALETTE_COLORS} colors",
            material_id.to_u32()
        )));
    }
    Ok(cell as u8 + 1)
}

/// The Voxel Max material slot material `material_id` draws: the one value id
/// its material-axis properties share, the material byte the loader set. Zero
/// when the palette binds no material axis. Errors when the properties
/// disagree, because the material is then no single slot, and when the slot
/// reaches [`MATERIAL_SLOTS`].
fn material_slot(layout: &PaletteLayout, material_id: U32Id<BVoxMaterial>) -> Result<u8> {
    let mut slot: Option<(u32, &str)> = None;
    for property in &layout.material {
        let value_id = layout
            .palette
            .value_id(material_id, property.id)
            .expect("a live material has a value id for every property")
            .to_u32();
        match slot {
            None => slot = Some((value_id, property.name)),

            Some((slot, _)) if slot == value_id => {}

            Some((slot, first)) => {
                return Err(Error::invalid(format!(
                    "material {} draws `{}` value {value_id} but `{first}` value {slot}, so it \
                     is no single Voxel Max material slot",
                    material_id.to_u32(),
                    property.name
                )));
            }
        }
    }
    let Some((slot, _)) = slot else {
        return Ok(0);
    };
    if slot as usize >= MATERIAL_SLOTS {
        return Err(Error::invalid(format!(
            "material {} draws slot {slot}, but a Voxel Max palette holds only {MATERIAL_SLOTS} \
             material slots",
            material_id.to_u32()
        )));
    }
    Ok(slot as u8)
}

/// Checks that material `material_id` looks the same on a slot glowing at
/// `sic`. Voxel Max glows in the voxel's base color at `sic`, while a voxcore
/// material glows in `emissiveColor` at `emissiveStrength`, so the two agree
/// only when the material's emissive color is its base color. A slot at zero
/// glows nowhere, whatever the colors. Errors when a glowing slot's material
/// has no emissive color, no base color, or an emissive that differs from its
/// base: Voxel Max would glow in a color the source never showed.
fn check_emissive(
    layout: &PaletteLayout,
    material_id: U32Id<BVoxMaterial>,
    sic: f64,
) -> Result<()> {
    if sic == 0.0 {
        return Ok(());
    }
    let color = |property: Option<&ColorProperty>, name: &str| -> Result<[f64; 3]> {
        let Some(property) = property else {
            return Err(Error::invalid(format!(
                "material {} sits in a slot glowing at {sic}, but the palette binds no `{name}` \
                 for Voxel Max to glow in",
                material_id.to_u32()
            )));
        };
        let value_id = layout
            .palette
            .value_id(material_id, property.id)
            .expect("a live material has a value id for every property");
        let color = property
            .colors
            .lin_srgba_f64(value_id)
            .expect("a live material draws one of its property's values");

        Ok([color.red, color.green, color.blue])
    };
    let emissive = color(layout.emissive_color.as_ref(), EMISSIVE_COLOR)?;
    let base = color(layout.color.as_ref(), BASE_COLOR)?;
    if emissive != base {
        return Err(Error::invalid(format!(
            "material {} glows in linear {emissive:?} over a base color of linear {base:?}, but \
             Voxel Max glows only in the base color, so writing it would change how the model \
             looks",
            material_id.to_u32()
        )));
    }
    Ok(())
}

/// The entry for an entity, or the error for an ext out of step with the
/// scene.
fn ext_entry<'a, T>(entry: Option<&'a T>, what: &str, id: u32) -> Result<&'a T> {
    entry.ok_or_else(|| Error::invalid(format!("vmax ext holds no entry for {what} {id}")))
}

/// The plan every object with no layer shares. It borrows the default palette
/// name because Voxel Max cannot resolve an empty `pal`, and writes no file.
fn colorless_plan() -> PalettePlan {
    PalettePlan {
        pal: FALLBACK_PALETTE.to_owned(),
        name: String::new(),
        color_table: None,
        indices: BTreeMap::new(),
        materials: Vec::new(),
    }
}

/// The filename suffix for an object: empty for object 0, then its numeric id.
fn object_file_suffix(object_id: U32Id<BVoxObject>) -> String {
    let index = object_id.to_u32();
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}

/// The per-object ext for an extra object on a node placing several, such as a
/// Goxel layer's blocks. It takes a distinct id and a distinct index triplet
/// and inherits the node's rotation and alignment to stay a sibling of the
/// node's first object. Its content box is derived from its own bounds on
/// write.
fn secondary_object_ext(
    node_ext: &VMaxExtNode,
    slot: usize,
    used_indices: &mut HashSet<[i64; 3]>,
) -> VMaxExtNode {
    let index = (0..)
        .map(|counter| [0, 0, counter])
        .find(|index| !used_indices.contains(index))
        .expect("a fresh counter exists");
    used_indices.insert(index);
    VMaxExtNode {
        id: secondary_uuid(&node_ext.id, slot),
        index,
        ..node_ext.clone()
    }
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

/// Re-bases the tight object's voxels to absolute model space, each with the
/// indices its material takes in `plan`. A voxel of an object with no layer
/// takes cell 1, since 0 is the empty cell, and slot 0. Errors when the object
/// has a second layer.
fn reconstruct_voxels(
    object: &VoxObject,
    plan: &PalettePlan,
    box_min: [i32; 3],
) -> Result<Vec<VMaxVoxel>> {
    let layer = object_layer(object)?;
    Ok(object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let indices = match layer {
                Some((layer_id, _)) => {
                    let material_id = object
                        .voxel_material(voxel_id, layer_id)
                        .expect("a live voxel samples its layer");
                    plan.indices[&material_id]
                }

                None => VoxelIndices {
                    color_idx: 1,
                    material_idx: 0,
                },
            };
            VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx: indices.material_idx,
                color_idx: indices.color_idx,
            }
        })
        .collect())
}

/// The editor state a contents file carries, without its snapshots: the
/// entry's session as it is, with the canvas `vp` re-scoped to the derived
/// build volume.
fn contents_editor_state(
    object_state: &VMaxExtObjectState,
    placement: &ObjectPlacement,
) -> VMaxContentsVmaxbFile {
    let mut tools = object_state.tools.clone();
    if let Some(tools) = tools.as_mut() {
        tools.vp = Some(placement.view_box.clone());
    }
    VMaxContentsVmaxbFile {
        snapshots: Vec::new(),
        uuid: object_state.uuid.clone(),
        v: object_state.v,
        tools,
        brush: object_state.brush.clone(),
        cam: object_state.cam.clone(),
        pal: None,
    }
}

/// The scene object for a node called `name`, which `transform` places on
/// Voxel Max's Z-up axes.
#[allow(clippy::too_many_arguments)]
fn object_from_node(
    name: &str,
    transform: &TyTransformF64,
    ext_node: &VMaxExtNode,
    parent_id: Option<String>,
    rotation: [f64; 4],
    placement: &ObjectPlacement,
    data: String,
    pal: String,
    suffix: &str,
) -> VMaxObject {
    VMaxObject {
        name: name.to_owned(),
        data,
        palette: pal,
        history: format!("history{suffix}.vmaxhb"),
        id: ext_node.id.clone(),
        parent_id,
        hidden: None,
        position: unbake_position(transform, decode_axis_angle(rotation), placement),
        rotation,
        scale: transform.scale.to_array(),
        ind: ext_node.index,
        s: ext_node.selected,
        t_al: ext_node.alignment.clone(),
        t_pa: ext_node.pivot_align.clone(),
        t_pf: ext_node.pivot_face.clone(),
        t_po: None,
        center: placement.center,
        bounds_min: Some(placement.bounds_min),
        bounds_max: Some(placement.bounds_max),
    }
}

/// Recovers an object's `t_p`, the inverse of the read path's
/// `object_transform`. It backs out the `t_p` Voxel Max renders with from the
/// node's transform, the content center it pivots about, and the grid `origin`:
/// `t_p = position - center - R*S* (box_min - center - origin)`. Uses the
/// axis-angle the object writes, so the two stay exact inverses.
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
    let rotated = rotation * offset;
    [
        transform.position.x - center[0] - rotated.x,
        transform.position.y - center[1] - rotated.y,
        transform.position.z - center[2] - rotated.z,
    ]
}

/// Writes each colored plan's color image and material sidecar.
fn write_palette_files(
    plans: &[PalettePlan],
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    vmax_color_format: VMaxColorFormat,
) -> Result<()> {
    for plan in plans {
        let Some(colors) = &plan.color_table else {
            continue;
        };
        let stem = plan
            .pal
            .strip_suffix(".png")
            .expect("a colored plan names a png");
        if matches!(
            vmax_color_format,
            VMaxColorFormat::Png | VMaxColorFormat::All
        ) {
            // 256 entries: 255 0-based colors then a transparent terminator.
            let mut cells = colors.clone();
            cells.push([0, 0, 0, 0]);
            palette_png_files.insert(plan.pal.clone(), VMaxPalettePngFile(cells));
        }
        // The settings sidecar carries the materials, and the colors when no
        // image does. Plist mode writes no image, so even a color-only object
        // writes its colors here rather than dropping them.
        let write_sidecar =
            !plan.materials.is_empty() || matches!(vmax_color_format, VMaxColorFormat::Plist);
        if write_sidecar {
            let sidecar = format!("{stem}.settings.vmaxpsb");
            // The plist `colors` table is the 255 colors with no terminator.
            let sidecar_colors = match vmax_color_format {
                VMaxColorFormat::Png => Vec::new(),
                VMaxColorFormat::Plist | VMaxColorFormat::All => colors.clone(),
            };
            // The per-color material map Voxel Max renders from: each used
            // color cell carries a bit for the material it draws.
            let (lc, indices, current) = color_material_map(plan);
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
    }
    Ok(())
}

/// The per-color material map for a palette's settings sidecar. For each color
/// cell a material draws, sets its slot's bit in `lc` (`1 << material_idx`),
/// lists the used cells in `indices`, and takes the first as `current`. Empty
/// for a palette with no materials, which Voxel Max renders with its own
/// defaults. Voxel Max reads a voxel's material from this map, not the
/// per-voxel byte.
fn color_material_map(plan: &PalettePlan) -> (Vec<u8>, Vec<i64>, i64) {
    let mut lc = vec![0u8; 256];
    if plan.materials.is_empty() {
        return (lc, Vec::new(), 0);
    }
    let mut cells: BTreeSet<u32> = BTreeSet::new();
    for indices in plan.indices.values() {
        let cell = u32::from(indices.color_idx) - 1;
        lc[cell as usize] |= 1 << indices.material_idx;
        cells.insert(cell);
    }
    let indices: Vec<i64> = cells.iter().map(|&cell| i64::from(cell)).collect();
    let current = indices.first().copied().unwrap_or(0);
    (lc, indices, current)
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

/// A coefficient snapped to an f32-exact value. Voxel Max decodes material
/// coefficients as 32-bit floats and drops a whole palette whose coefficients
/// are not f32-representable, so every one passes through here.
fn to_f32(value: f64) -> f64 {
    f64::from(value as f32)
}

/// Applies the scene camera choice to the rebuilt scene. `Ext` leaves the
/// camera the ext supplied.
fn apply_scene_camera(scene: &mut VMaxSceneJsonFile, scene_camera: SceneCameraSource) {
    match scene_camera {
        SceneCameraSource::Ext => {}
        SceneCameraSource::Empty => scene.cam = Some(SYNTH_CAMERA),
        SceneCameraSource::Camera(camera) => scene.cam = Some(camera),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        SceneCameraSource, VMaxExtNode, VMaxVoxMain, VMaxWriteOptions, from_vmax_file,
        to_vmax_file, to_vmax_vox_main,
    };
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3I32, TyVector3U32};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion,
        VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
        VMaxSceneJsonFile, VMaxViewBox,
        snapshots::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots},
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePoolValue,
        VoxHierarchyNode, VoxObject,
        material::{BASE_COLOR, EMISSIVE_COLOR, ROUGHNESS},
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

    /// A coefficient snapped to an f32-exact value, matching what the writer
    /// stores so a fixture round-trips to equality.
    fn f32r(value: f64) -> f64 {
        f64::from(value as f32)
    }

    /// The neutral default material the writer pads unused slots with.
    fn default_material(slot: usize) -> VMaxMaterial {
        VMaxMaterial {
            mi: (slot + 1).to_string(),
            mc: f32r(0.1),
            rc: f32r(0.9),
            sic: 0.0,
            sh: true,
            tc: None,
            md: None,
        }
    }

    /// A 256-byte `lc` usage mask with `1 << material_byte` set at each
    /// `(color_cell, material_byte)`.
    fn lc_mask(cells: &[(usize, u8)]) -> Vec<u8> {
        let mut lc = vec![0u8; 256];
        for &(cell, byte) in cells {
            lc[cell] |= 1 << byte;
        }
        lc
    }

    /// The fixed settings sidecar the reverse path writes on a rebuilt material
    /// palette: the real materials padded to the eight slots, the per-color
    /// material map (`lc`/`indices`/`current`), and the default editor state,
    /// so a fixture round-trips to equality.
    fn palette_settings(
        name: &str,
        materials: Vec<VMaxMaterial>,
        colors: Vec<[u8; 4]>,
        lc: Vec<u8>,
        indices: Vec<i64>,
        current: i64,
    ) -> VMaxPaletteSettingsVmaxpsbFile {
        let mut materials = materials;
        if !materials.is_empty() {
            while materials.len() < 8 {
                materials.push(default_material(materials.len()));
            }
        }
        VMaxPaletteSettingsVmaxpsbFile {
            name: name.to_owned(),
            materials,
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

    /// A 256-cell image: 255 colors then a transparent terminator.
    fn palette_png() -> VMaxPalettePngFile {
        let mut cells: Vec<[u8; 4]> = (0..255u32).map(|i| [i as u8, 0, 0, 255]).collect();
        cells.push([0, 0, 0, 0]);
        VMaxPalettePngFile(cells)
    }

    /// A document with a root group, a child object placed at the origin with
    /// authored bounds, a shared color and material palette, and preserved
    /// scene and object editor state. Built in the canonical form the reverse
    /// path emits so it round-trips to equality.
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
            // The group's content box is derived from its subtree: the child
            // object sits at node-local [0, 0, 0]..[2, 2, 2], so the box
            // centers on [1, 1, 1] with unit half-extents.
            center: [1.0, 1.0, 1.0],
            bounds_min: Some([-1.0, -1.0, -1.0]),
            bounds_max: Some([1.0, 1.0, 1.0]),
        };
        let object = VMaxObject {
            name: "obj".to_owned(),
            data: "contents.vmaxb".to_owned(),
            palette: "palette1.png".to_owned(),
            history: "history.vmaxhb".to_owned(),
            id: "o".to_owned(),
            parent_id: Some("g".to_owned()),
            hidden: None,
            // Canonical placement the reverse path emits: the grid is centered
            // in the workspace, the content box is symmetric about the center,
            // and the placement pivots about it.
            position: [-127.0, -127.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: Some(false),
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [128.0, 128.0, 1.0],
            bounds_min: Some([-1.0, -1.0, -1.0]),
            bounds_max: Some([1.0, 1.0, 1.0]),
        };

        let scene_json_file = VMaxSceneJsonFile {
            v: 4,
            cam: Some(VMaxSceneCamera::default()),
            background: Some("#101010".to_owned()),
            groups: vec![group],
            objects: vec![object],
            ..Default::default()
        };

        // Canonical snapshots: voxels re-encoded at the centered internal grid
        // just as the reverse path emits, the runtime grid [127, 127, 0]..[128,
        // 128, 1].
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[
                VMaxVoxel {
                    position: [127, 127, 0],
                    material_idx: 0,
                    color_idx: 5,
                },
                VMaxVoxel {
                    position: [128, 128, 1],
                    material_idx: 1,
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
            "palette1.settings.vmaxpsb".to_owned(),
            palette_settings(
                "mat",
                vec![
                    material("1", 0.0, 1.0, 0.0, true),
                    material("2", 0.5, 0.25, 2.0, false),
                ],
                Vec::new(),
                // Voxel 0 (color cell 4) draws material 0; voxel 1 (cell 2)
                // draws material 1.
                lc_mask(&[(4, 0), (2, 1)]),
                vec![2, 4],
                2,
            ),
        );
        let mut palette_png_files = BTreeMap::new();
        palette_png_files.insert("palette1.png".to_owned(), palette_png());

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

    /// `sample()` with two more objects under the group, each with its own
    /// contents file and editor state.
    fn three_object_sample() -> VMaxFile {
        let mut file = sample();
        let object = file.scene_json_file.objects[0].clone();
        let contents = file.contents_files["contents.vmaxb"].clone();
        for (id, suffix) in [("o2", "2"), ("o3", "3")] {
            let data = format!("contents{suffix}.vmaxb");
            file.scene_json_file.objects.push(VMaxObject {
                name: id.to_owned(),
                data: data.clone(),
                history: format!("history{suffix}.vmaxhb"),
                id: id.to_owned(),
                ..object.clone()
            });
            file.contents_files.insert(
                data,
                VMaxContentsVmaxbFile {
                    uuid: format!("u{suffix}"),
                    ..contents.clone()
                },
            );
        }
        file
    }

    /// Releasing a middle node, its object, and the object's palette, then
    /// compacting, drops their entries and rekeys the survivors' to their
    /// compacted ids. The rebuilt document carries the survivors' ids and
    /// editor state, and reloads to the same ext.
    #[test]
    fn released_entities_leave_the_survivors_provenance_rekeyed() {
        let file = three_object_sample();
        let mut main = from_vmax_file(&file).unwrap();
        let original = main.ext().clone();

        // The group, node 0, places nodes 1..=3. Node 2 places object 1, which
        // draws palette 1.
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let doomed_node_id = U32Id::<BVoxHierarchyNode>::from_u32(2);
        let doomed_object_id = U32Id::<BVoxObject>::from_u32(1);
        let doomed_palette_id = U32Id::<BVoxPalette>::from_u32(1);
        let mut group = main.hierarchy_node(group_id).unwrap().clone();
        group.child_node_ids.retain(|&id| id != doomed_node_id);
        main.set_hierarchy_node_children(group_id, group.child_node_ids, group.child_object_ids)
            .unwrap();
        main.release_hierarchy_node(doomed_node_id).unwrap();
        main.release_object(doomed_object_id).unwrap();
        main.release_palette(doomed_palette_id).unwrap();
        main.gc().unwrap();

        // The last node, object, and palette each slide down one id.
        let mut expected = original;
        expected.hierarchy_nodes.remove(&doomed_node_id);
        let last_node = expected
            .hierarchy_nodes
            .remove(&U32Id::from_u32(3))
            .unwrap();
        expected.hierarchy_nodes.insert(doomed_node_id, last_node);
        expected.object_states.remove(&doomed_object_id);
        let last_object = expected.object_states.remove(&U32Id::from_u32(2)).unwrap();
        expected.object_states.insert(doomed_object_id, last_object);
        expected.palettes.remove(&doomed_palette_id);
        let last_palette = expected.palettes.remove(&U32Id::from_u32(2)).unwrap();
        expected.palettes.insert(doomed_palette_id, last_palette);
        assert_eq!(main.ext(), &expected);

        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let ids: Vec<&str> = rebuilt
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect();
        assert_eq!(ids, ["o", "o3"]);
        let uuids: BTreeSet<&str> = rebuilt
            .contents_files
            .values()
            .map(|contents| contents.uuid.as_str())
            .collect();
        assert_eq!(uuids, BTreeSet::from(["u", "u3"]));

        let reloaded = from_vmax_file(&rebuilt).unwrap();
        assert_eq!(reloaded.ext(), &expected);
    }

    /// A node and an object retained after the load take synthesized entries
    /// as they are retained. The write reads them back out and links no
    /// parent for a root.
    #[test]
    fn a_node_retained_after_the_load_takes_a_synthesized_entry() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let mut object = VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        let voxel_id = object.voxel_id(TyVector3U32::splat(0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::<BVoxMaterial>::from_u32(0)])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "added".to_owned(),
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let ext = main.ext();
        assert_eq!(ext.hierarchy_nodes.len(), 3);
        assert_eq!(
            ext.hierarchy_nodes[&node_id],
            VMaxExtNode {
                id: "00000000-0000-0000-0000-000000000001".to_owned(),
                index: [0, 0, 1],
                rotation: [0.0, 0.0, 0.0, 0.0],
                alignment: "f".to_owned(),
                pivot_face: "8".to_owned(),
                pivot_align: "4".to_owned(),
                selected: None,
            }
        );
        assert_eq!(ext.object_states.len(), 2);
        let object_state = &ext.object_states[&object_id];
        assert_eq!(object_state.uuid, "00000000-0000-0001-0000-000000000001");
        assert_eq!(object_state.v, 4);
        assert!(object_state.tools.is_some() && object_state.brush.is_some());
        assert_eq!(
            object_state.cam.as_ref().map(|cam| cam.o),
            Some([127.5, 127.5, 0.5])
        );

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let added = file
            .scene_json_file
            .objects
            .iter()
            .find(|object| object.name == "added")
            .expect("the retained node writes as an object");
        assert_eq!(added.id, "00000000-0000-0000-0000-000000000001");
        assert_eq!(added.ind, [0, 0, 1]);
        assert_eq!(added.parent_id, None);
        assert_eq!(added.t_al, "f");
        let contents = &file.contents_files[&added.data];
        assert_eq!(contents.uuid, "00000000-0000-0001-0000-000000000001");
        assert_eq!(
            contents.tools.as_ref().and_then(|tools| tools.vp.clone()),
            Some(VMaxViewBox {
                min: [127, 127, 0],
                max: [127, 127, 0],
            })
        );

        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(reloaded.ext().hierarchy_nodes.len(), 3);
    }

    /// A voxel remap moves the object's camera target by as much as the
    /// content center moved, and keeps its uuid.
    #[test]
    fn a_voxel_remap_moves_the_camera_target_with_the_content() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let mut object = VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        object
            .retain_voxel(U32Id::from_u32(0), &[U32Id::<BVoxMaterial>::from_u32(0)])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();
        let uuid = main.ext().object_states[&object_id].uuid.clone();

        // The canvas widens by 2 and recenters 1 lower, and the voxel moves 2
        // up, so the content center moves 1 along x.
        main.remap_object_voxels(object_id, TyVector3U32::new(3, 1, 1), |p| {
            p.as_ivec3() + TyVector3I32::new(2, 0, 0)
        })
        .unwrap();

        let object_state = &main.ext().object_states[&object_id];
        assert_eq!(
            object_state.cam.as_ref().map(|cam| cam.o),
            Some([128.5, 127.5, 0.5])
        );
        assert_eq!(object_state.uuid, uuid);
    }

    /// A voxel resample moves the object's camera target by as much as the
    /// content center moved, as a remap does.
    #[test]
    fn a_voxel_resample_moves_the_camera_target_with_the_content() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let mut object = VoxObject::new(String::new(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
        object
            .retain_voxel(U32Id::from_u32(0), &[U32Id::<BVoxMaterial>::from_u32(0)])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();

        // The same regrid the remap test makes: the canvas widens by 2 and
        // the one voxel lands 2 up, so the content center moves 1 along x.
        main.resample_object_voxels(object_id, TyVector3U32::new(3, 1, 1), |p| {
            (p.x == 2).then_some(TyVector3U32::ZERO)
        })
        .unwrap();

        let object_state = &main.ext().object_states[&object_id];
        assert_eq!(
            object_state.cam.as_ref().map(|cam| cam.o),
            Some([128.5, 127.5, 0.5])
        );
    }

    /// A node rotated after the load writes its live rotation on Voxel Max's
    /// Z-up axes, a turn about voxcore's `+y` landing on `+z`. An unrotated
    /// node keeps the preserved spelling.
    #[test]
    fn a_node_rotated_after_the_load_writes_its_live_rotation() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let mut group = main.hierarchy_node(group_id).unwrap().clone();
        group.transform.rotation = TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 0.5);
        main.set_hierarchy_node_transform(group_id, group.transform)
            .unwrap();

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let [x, y, z, angle] = file.scene_json_file.groups[0].rotation;
        assert!(x.abs() < 1e-12 && y.abs() < 1e-12);
        assert!((z - 1.0).abs() < 1e-12);
        assert!((angle - 0.5).abs() < 1e-12);
        assert_eq!(
            file.scene_json_file.objects[0].rotation,
            [0.0, 0.0, 0.0, 0.0]
        );
    }

    /// Voxel Max holds a tree, so a node placed by two parents, or a root
    /// that is also a child, refuses to write instead of picking a parent.
    #[test]
    fn a_node_with_two_parents_or_a_root_child_errors() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let object_node_id = U32Id::<BVoxHierarchyNode>::from_u32(1);
        let other_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "other".to_owned(),
                child_node_ids: vec![object_node_id],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(other_id).unwrap();
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());

        let mut main = from_vmax_file(&sample()).unwrap();
        main.push_root_hierarchy_node_id(object_node_id).unwrap();
        assert!(main.root_hierarchy_node_ids().contains(&group_id));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());
    }

    /// A material retained after the load draws the slot its material-axis
    /// value ids share, and the writer puts its voxels there.
    #[test]
    fn a_retained_material_draws_the_slot_its_values_share() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        // Color cell 0 in slot 1: every material-axis property takes value 1.
        let value_ids: Vec<U32Id<BVoxValuePoolValue>> = main
            .palette(palette_id)
            .unwrap()
            .iter_properties()
            .map(|(_, property)| {
                U32Id::from_u32(u32::from(
                    property.name != BASE_COLOR && property.name != EMISSIVE_COLOR,
                ))
            })
            .collect();
        let material_id = main.retain_material(palette_id, value_ids).unwrap();
        let voxel_id = main
            .object(object_id)
            .unwrap()
            .voxel_id(TyVector3U32::splat(0))
            .unwrap();
        main.retain_voxel(object_id, voxel_id, &[material_id])
            .unwrap();

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let voxels =
            decode_vmax_snapshots(&file.contents_files["contents.vmaxb"].snapshots).unwrap();
        assert!(
            voxels
                .iter()
                .any(|voxel| voxel.color_idx == 1 && voxel.material_idx == 1)
        );
        let settings = &file.palette_settings_files["palette1.settings.vmaxpsb"];
        assert_eq!(settings.lc[0], 0b10);
    }

    /// A material whose material-axis properties draw different values is no
    /// single Voxel Max slot, so the writer errors whether or not a voxel
    /// draws it.
    #[test]
    fn a_material_drawing_two_slots_errors() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        let value_ids: Vec<U32Id<BVoxValuePoolValue>> = main
            .palette(palette_id)
            .unwrap()
            .iter_properties()
            .map(|(_, property)| U32Id::from_u32(u32::from(property.name == ROUGHNESS)))
            .collect();
        main.retain_material(palette_id, value_ids).unwrap();

        let error = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap_err();
        assert!(error.to_string().contains("single"), "{error}");
    }

    /// Voxel Max lists child groups before their parents in its own files:
    /// in `tyt-assets/src/vmax/mixed-shapes.vmax/scene.json` the groups at
    /// index 0 and 1 have a `pid` naming the group at index 2. The loader and
    /// the writer keep that listing order.
    #[test]
    fn child_groups_listed_before_their_parent_round_trip_in_order() {
        let mut file = sample();
        let root = file.scene_json_file.groups[0].clone();
        let group = |id: &str, parent_id: Option<&str>, ind: [i64; 3]| VMaxGroup {
            id: id.to_owned(),
            parent_id: parent_id.map(str::to_owned),
            ind,
            ..root.clone()
        };
        // The listing from the asset, with the object under the innermost
        // group so every group has geometry.
        file.scene_json_file.groups = vec![
            group(
                "233A8A71-7B3C-48D0-B0A8-D774275FA80E",
                Some("528B9E32-71C7-4887-9570-D920B7D9C988"),
                [0, 1, 0],
            ),
            group(
                "528B9E32-71C7-4887-9570-D920B7D9C988",
                Some("DEB88339-5C59-4CC6-B7F6-1DAC39397C1B"),
                [0, 1, 1],
            ),
            group("DEB88339-5C59-4CC6-B7F6-1DAC39397C1B", None, [0, 1, 2]),
        ];
        file.scene_json_file.objects[0].parent_id =
            Some("233A8A71-7B3C-48D0-B0A8-D774275FA80E".to_owned());

        let main = from_vmax_file(&file).unwrap();
        assert_eq!(main.root_hierarchy_node_ids(), [U32Id::from_u32(2)]);
        assert_eq!(
            main.hierarchy_node(U32Id::from_u32(2))
                .unwrap()
                .child_node_ids,
            [U32Id::from_u32(1)]
        );

        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(rebuilt.scene_json_file.groups, file.scene_json_file.groups);
        assert_eq!(
            rebuilt.scene_json_file.objects[0].parent_id,
            file.scene_json_file.objects[0].parent_id
        );
    }

    /// A reduction repaints onto a survivor, releases the rest, prunes the
    /// value pools, and compacts. The exact material list follows the pools,
    /// so the survivor keeps its exact material at its compacted value's slot,
    /// and the reload records the list padded back out.
    #[test]
    fn a_reduction_keeps_the_survivors_exact_material_through_prune_and_gc() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // The palette's materials sort by color cell then slot: material 0 draws
        // cell 2 in slot 1 and material 1 draws cell 4 in slot 0. Keep material
        // 0, whose slot's values renumber to 0 once slot 0's are pruned away.
        let doomed_id = U32Id::<BVoxMaterial>::from_u32(1);
        let survivor_id = U32Id::<BVoxMaterial>::from_u32(0);
        main.repaint_materials(palette_id, &HashMap::from([(doomed_id, survivor_id)]))
            .unwrap();
        main.release_material(palette_id, doomed_id).unwrap();
        main.prune_value_pools();
        main.gc().unwrap();
        let exact = &main.ext().palettes[&palette_id].materials;
        assert_eq!(exact.len(), 1);
        assert_eq!(exact[0].emissive, 2.0);

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let voxels =
            decode_vmax_snapshots(&file.contents_files["contents.vmaxb"].snapshots).unwrap();
        assert!(voxels.iter().all(|voxel| voxel.material_idx == 0));
        let settings = &file.palette_settings_files["palette1.settings.vmaxpsb"];
        assert_eq!(settings.materials[0], material("1", 0.5, 0.25, 2.0, false));

        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(reloaded.ext().palettes[&palette_id].materials.len(), 8);
    }

    /// An ext missing an entity's entry is malformed, so the writer errors
    /// instead of synthesizing one.
    #[test]
    fn an_ext_missing_an_entry_errors() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let ext = main.ext_mut();
        ext.hierarchy_nodes.remove(&U32Id::from_u32(1));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());

        let mut main = from_vmax_file(&sample()).unwrap();
        let ext = main.ext_mut();
        ext.object_states.remove(&U32Id::from_u32(0));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());

        let mut main = from_vmax_file(&sample()).unwrap();
        let ext = main.ext_mut();
        ext.palettes.remove(&U32Id::from_u32(0));
        assert!(to_vmax_file(&main, &VMaxWriteOptions::default()).is_err());
    }

    #[test]
    fn round_trips_through_vox_state() {
        let original = sample();
        let main = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(rebuilt, original);
    }

    /// A document written back through a bare state, its ext dropped, reads
    /// its materials from the value pools the loader built, one per slot, so
    /// the material list keeps its order and every voxel keeps its cell and
    /// slot however many colors a glowing slot spans.
    #[test]
    fn writes_back_from_its_pools_without_the_ext() {
        let mut original = sample();
        let voxels: Vec<VMaxVoxel> = (0..10u8)
            .map(|i| VMaxVoxel {
                position: [127 + i32::from(i), 127, 0],
                material_idx: i % 2,
                color_idx: 10 + i,
            })
            .collect();
        original
            .contents_files
            .get_mut("contents.vmaxb")
            .unwrap()
            .snapshots = encode_vmax_snapshots(&voxels);
        let main = from_vmax_file(&original).unwrap();
        let bare = to_vmax_vox_main(main.take_ext().main).unwrap();
        let rebuilt = to_vmax_file(&bare, &VMaxWriteOptions::default()).unwrap();

        assert_eq!(rebuilt.palette_settings_files.len(), 1);
        let settings = rebuilt.palette_settings_files.values().next().unwrap();
        let sics: Vec<f64> = settings.materials.iter().map(|m| m.sic).collect();
        assert_eq!(sics, [0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // The bare path re-seats the grid, so compare the cells and slots in
        // row order rather than the positions.
        let mut written =
            decode_vmax_snapshots(&rebuilt.contents_files.values().next().unwrap().snapshots)
                .unwrap();
        written.sort_by_key(|v| v.position);
        let samples = |voxels: &[VMaxVoxel]| -> Vec<(u8, u8)> {
            voxels
                .iter()
                .map(|v| (v.material_idx, v.color_idx))
                .collect()
        };
        assert_eq!(samples(&written), samples(&voxels));
    }

    /// A material carrying dispersion, a transmission color (`tc`), and a
    /// non-finite coefficient round-trips exactly, alongside a plain material
    /// and the padded default slots. The value pools carry a neutral copy, but
    /// the ext keeps the exact values, so the rebuilt document matches byte for
    /// byte. Coefficients are f32-exact, as Voxel Max stores them.
    #[test]
    fn round_trips_rich_materials() {
        let mut original = sample();
        let mut materials = vec![
            VMaxMaterial {
                mi: "1".to_owned(),
                mc: f64::INFINITY,
                rc: f32r(0.25),
                sic: f32r(1.0),
                sh: true,
                tc: Some(f32r(0.6)),
                md: Some(VMaxMaterialDispersion {
                    absorption: f32r(0.1),
                    ior: f32r(1.4),
                    transmission: f32r(0.3),
                }),
            },
            material("2", f32r(0.5), f32r(0.25), f32r(2.0), false),
        ];
        for slot in materials.len()..8 {
            materials.push(default_material(slot));
        }
        original
            .palette_settings_files
            .get_mut("palette1.settings.vmaxpsb")
            .unwrap()
            .materials = materials;
        let main = from_vmax_file(&original).unwrap();
        let rebuilt = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(rebuilt, original);
    }

    #[test]
    fn nan_material_coefficient_is_an_error() {
        let mut original = sample();
        let mut materials = vec![material("1", f32r(0.5), f32r(0.25), f64::NAN, false)];
        for slot in materials.len()..8 {
            materials.push(default_material(slot));
        }
        original
            .palette_settings_files
            .get_mut("palette1.settings.vmaxpsb")
            .unwrap()
            .materials = materials;
        assert!(from_vmax_file(&original).is_err());
    }

    /// Puts a scene camera in the state's ext for the typed path to keep or
    /// replace.
    fn with_ext_scene_camera(main: &mut VMaxVoxMain, cam: VMaxSceneCamera) {
        let vmax_ext = main.ext_mut();
        vmax_ext.scene.cam = Some(cam);
    }

    #[test]
    fn scene_camera_ext_keeps_the_ext_camera() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let cam = VMaxSceneCamera {
            z: 321.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut main, cam);
        let options = VMaxWriteOptions {
            scene_camera: SceneCameraSource::Ext,
            ..Default::default()
        };
        let file = to_vmax_file(&main, &options).unwrap();
        assert_eq!(file.scene_json_file.cam, Some(cam));
    }

    /// `SceneCameraSource::Empty` replaces the ext's scene camera with the
    /// default, while leaving the camera unset keeps it.
    #[test]
    fn scene_camera_empty_replaces_the_ext_camera() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let cam = VMaxSceneCamera {
            z: 999.0,
            ..Default::default()
        };
        with_ext_scene_camera(&mut main, cam);

        let kept = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        assert_eq!(kept.scene_json_file.cam, Some(cam));

        let options = VMaxWriteOptions {
            scene_camera: SceneCameraSource::Empty,
            ..Default::default()
        };
        let empty = to_vmax_file(&main, &options).unwrap();
        assert_ne!(empty.scene_json_file.cam, Some(cam));
    }
}
