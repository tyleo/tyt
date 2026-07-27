use crate::{
    ABSORPTION, BASE_COLOR_FACTOR, EMISSIVE_FACTOR, EMISSIVE_STRENGTH, Error, IOR, METALLIC_FACTOR,
    ROUGHNESS_FACTOR, Result, SHADOWS, TRANSMISSION_FACTOR, VoxelMaxExt, VoxelMaxExtWrapper,
    VoxelMaxMaterial, VoxelMaxMaterialDispersion, VoxelMaxNode, VoxelMaxObjectState,
    VoxelMaxPalette, default_scalar, to_vox_value, vm_coefficient_to_pbr_factor,
};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{
    TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32,
    ZERO_LENGTH_TOLERANCE,
};
use vmax::{
    VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxObject,
    VMaxSceneJsonFile, VMaxViewBox,
};
use vmax_codec::{VMaxVoxel, decode_vmax_snapshots};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePool, VoxBound,
    VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
};

/// Usable colors in a Voxel Max palette. Color indices are 1-based: `color_idx`
/// runs 1..=255 and 0 is the empty cell. Colors are stored 0-based; a
/// `palette*.png` is 256x1 RGBA with the colors at 0..254 and a transparent
/// terminator at 255, while the plist `colors` table is just the 255 colors.
const COLOR_CELLS: usize = 255;

/// The color used for every entry of a placeholder color table when an object's
/// colors are missing, so the color indices are still preserved.
const PLACEHOLDER_COLOR: [u8; 4] = [255, 255, 255, 255];

/// Loads a Voxel Max document into a [`VoxMain`].
///
/// Geometry, palettes, and hierarchy become native voxcore entities. The Voxel
/// Max state with no native voxcore home rides in a `voxel-max` ext so the
/// document can be written back faithfully. Voxel snapshots are decoded to
/// voxels on the fly and palette color tables unpacked as needed. Color indices
/// are 1-based in Voxel Max, so a voxel's color cell is `color_idx - 1`; the
/// material byte is 0-based and used directly.
///
/// Errors on malformed geometry or if
/// [`VoxMain::validate`](voxcore::VoxMain::validate) rejects the result.
pub fn from_vmax_file(serde: &VMaxFile) -> Result<VoxMain> {
    let scene = &serde.scene_json_file;
    let mut state = VoxMain::default();

    // One folded palette per distinct object, aligned by palette id with the
    // ext provenance carrying its name and exact material list. Instances reuse
    // an object, so they share its palette rather than deduping by source.
    let mut palette_provenance: Vec<Option<VoxelMaxPalette>> = Vec::new();

    // One voxcore object per distinct geometry; instances of one geometry
    // collapse to a single object placed by several nodes.
    let mut object_transforms: Vec<TyTransformF64> = Vec::new();
    let mut object_refs: Vec<usize> = Vec::new();
    let mut object_data: Vec<Option<String>> = Vec::new();
    let mut instances: HashMap<InstanceKey, usize> = HashMap::new();
    for object in &scene.objects {
        let key = instance_key(object);
        if let Some(&existing) = key.as_ref().and_then(|key| instances.get(key)) {
            // An instance shares the geometry it re-places, so it re-derives
            // only the placing transform from its own content box and pivot.
            let box_min = authored_box(object).map_or([0, 0, 0], |(box_min, _)| box_min);
            let origin = pivot_origin(box_min, object.center);
            object_transforms.push(object_transform(object, box_min, origin));
            object_refs.push(existing);
            continue;
        }
        let (vox_object, data, transform) =
            build_object(serde, object, &mut state, &mut palette_provenance)?;
        let object_id = state.add_object(vox_object);
        object_data.push(data);
        object_transforms.push(transform);
        object_refs.push(object_id.to_u32() as usize);
        if let Some(key) = key {
            instances.insert(key, object_id.to_u32() as usize);
        }
    }

    let (nodes, roots) = build_hierarchy(scene, &object_transforms, &object_refs);
    for node in nodes {
        state.add_hierarchy_node(node);
    }
    state.set_root_hierarchy_nodes(roots);

    // Each object's preserved editor state, aligned by object id with the
    // objects, read off the contents files.
    let object_states: Vec<Option<VoxelMaxObjectState>> = object_data
        .iter()
        .map(|data| {
            data.as_deref()
                .and_then(|data| serde.contents_files.get(data))
                .map(object_state_from_contents)
        })
        .collect();

    let voxel_max = voxel_max_ext(scene, palette_provenance, &object_states);
    let ext = to_vox_value(&VoxelMaxExtWrapper { voxel_max })?;
    state.set_ext(Some(ext));

    state.validate()?;
    Ok(state)
}

/// Captures the editor state of a contents file for the ext. The tool partition
/// (`tools.vp`) is dropped: it is the object's build volume, held natively as
/// the object's grid, so it is rebuilt on write rather than stored here.
fn object_state_from_contents(data: &VMaxContentsVmaxbFile) -> VoxelMaxObjectState {
    VoxelMaxObjectState {
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

/// Builds the voxcore object for one scene object, adding any palettes it
/// introduces to `state`. The object is built in its build volume (the author's
/// `tools.vp`), so its live voxels keep their authored positions inside it.
/// Returns the object, its backing `data` filename, and the transform of the
/// node that places it.
fn build_object(
    serde: &VMaxFile,
    object: &VMaxObject,
    state: &mut VoxMain,
    palette_provenance: &mut Vec<Option<VoxelMaxPalette>>,
) -> Result<(VoxObject, Option<String>, TyTransformF64)> {
    // Voxels come from decoding the object's snapshot edit-log on the fly.
    let voxels: Vec<VMaxVoxel> = if object.data.is_empty() {
        Vec::new()
    } else {
        decode_vmax_snapshots(&serde.contents_files[&object.data].snapshots)?
    };

    // The runtime grid is exactly tight: the occupied voxel extent, re-based so
    // the voxels fill it from the origin. An empty object is a degenerate [0,
    // 0, 0] grid seated at its content box so the placing node still pivots
    // about the recorded center. `origin` offsets that grid from the node so
    // the node transform pivots about the content center.
    let (box_min, bounds) = match min_corner(&voxels) {
        Some(min) => (min, object_bounds(&voxels, min)),
        // An empty object seats at its content box; lacking one, it seats at
        // the build volume `vp.min` so the edit grid still contains the runtime
        // grid, and only at the world origin when it has neither.
        None => (
            authored_box(object)
                .map(|(box_min, _)| box_min)
                .or_else(|| {
                    view_box(serde, object)
                        .map(|vp| [vp.min[0] as i32, vp.min[1] as i32, vp.min[2] as i32])
                })
                .unwrap_or([0, 0, 0]),
            [0, 0, 0],
        ),
    };
    let origin = pivot_origin(box_min, object.center);
    let transform = object_transform(object, box_min, origin);
    // The build volume is the author's `tools.vp`; its `origin` offsets it from
    // the placing node, and `offset` shifts a runtime-grid voxel into it.
    let (edit_bounds, edit_origin) = edit_grid(view_box(serde, object), box_min, origin, bounds);
    let offset = [
        origin[0] - edit_origin[0],
        origin[1] - edit_origin[1],
        origin[2] - edit_origin[2],
    ];
    let data = (!object.data.is_empty()).then(|| object.data.clone());

    let [size_x, size_y, size_z] = edit_bounds;
    let mut vox_object = VoxObject::new(
        object.name.clone(),
        TyVector3U32::new(size_x, size_y, size_z),
    )
    .ok_or_else(|| {
        invalid(format!(
            "object \"{}\" grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
            object.name,
            VoxObject::MAX_GRID_CELLS
        ))
    })?;
    vox_object.set_origin(TyVector3I32::new(
        edit_origin[0],
        edit_origin[1],
        edit_origin[2],
    ));

    if voxels.is_empty() {
        return Ok((vox_object, data, transform));
    }

    // Fold the color and material palettes into one: each voxel samples a
    // single material carrying both its color and its material coefficients, one
    // material per distinct color-and-material combination the voxels use.
    let folded = folded_palette(serde, object, &voxels, state);
    palette_provenance.push(Some(folded.provenance));

    // Back-fill the layer with material 0; the live voxels overwrite theirs.
    vox_object.add_layer(folded.palette, U32Id::<BVoxMaterial>::from_u32(0));

    for voxel in &voxels {
        // Shift the voxel from its model position into the build volume; an
        // out-of-grid result casts to a huge u32 and is rejected by `voxel_id`.
        let position = TyVector3U32::new(
            (voxel.position[0] - box_min[0] + offset[0]) as u32,
            (voxel.position[1] - box_min[1] + offset[1]) as u32,
            (voxel.position[2] - box_min[2] + offset[2]) as u32,
        );
        let voxel_id = vox_object.voxel_id(position).ok_or_else(|| {
            invalid(format!(
                "object \"{}\" voxel ({}, {}, {}) lies outside its build volume",
                object.name, voxel.position[0], voxel.position[1], voxel.position[2]
            ))
        })?;

        let material = folded.combos[&combo_key(voxel, folded.has_materials)];
        vox_object
            .retain_voxel(voxel_id, &[material])
            .ok_or_else(|| {
                invalid(format!(
                    "object \"{}\" has a malformed voxel sample",
                    object.name
                ))
            })?;
    }

    Ok((vox_object, data, transform))
}

/// A folded palette added to a [`VoxMain`], with the data `build_object` needs
/// to sample its voxels and record its ext provenance.
struct FoldedPalette {
    /// The palette id in the state.
    palette: U32Id<BVoxPalette>,

    /// The ext provenance carrying the name and exact material list.
    provenance: VoxelMaxPalette,

    /// Each used color-and-material combination's material id.
    combos: HashMap<(u8, u8), U32Id<BVoxMaterial>>,

    /// Whether the object carries materials.
    has_materials: bool,
}

/// Builds one folded palette for an object's live voxels, adding it and its
/// value pools to `state`.
///
/// The color pool is the object's full color table in order, so a material's
/// `baseColorFactor` value-index is `color_idx - 1`. Each material scalar pool
/// holds one value per Voxel Max material, in order; the material byte is
/// 0-based, so a voxel's value-index is its `material_idx`. The exact material
/// list rides in the ext provenance for a byte-exact write-back.
fn folded_palette(
    serde: &VMaxFile,
    object: &VMaxObject,
    voxels: &[VMaxVoxel],
    state: &mut VoxMain,
) -> FoldedPalette {
    let colors = color_cells(serde, object);
    let (name, materials) = material_list(serde, object);
    let has_materials = !materials.is_empty();

    // `baseColorFactor` and `emissiveFactor` read the voxel's color cell; every
    // material scalar reads its material byte. `color_axis` records each
    // property's axis in property order, so a folded material gathers
    // one value id per property.
    let mut palette = VoxPalette::default();
    let mut color_axis: Vec<bool> = Vec::new();
    let color_pool = state.add_value_pool(VoxValuePool::srgba(
        colors
            .iter()
            .map(|color| <[f64; 4]>::from(TySrgbaU8::from(*color).into_format::<f64, f64>()))
            .collect(),
    ));
    palette
        .add_property(BASE_COLOR_FACTOR.to_owned(), color_pool)
        .expect("the property names are distinct");
    color_axis.push(true);

    // Metalness and roughness convert from Voxel Max's 0.1 to 0.9 slider
    // coefficient to the 0 to 1 glTF factor the pool name implies; see
    // [`vm_coefficient_to_pbr_factor`]. The remaining scalars are unbounded and
    // stay raw: `sic` is an unbounded emission strength, and `shadows` and
    // `absorption` have no glTF counterpart. The exact coefficients ride in the
    // ext for a byte-exact write-back.
    if has_materials {
        let metallic = float_pool(
            state,
            materials
                .iter()
                .map(|m| vm_coefficient_to_pbr_factor(m.mc))
                .collect(),
        );
        palette
            .add_property(METALLIC_FACTOR.to_owned(), metallic)
            .expect("the property names are distinct");
        color_axis.push(false);
        let roughness = float_pool(
            state,
            materials
                .iter()
                .map(|m| vm_coefficient_to_pbr_factor(m.rc))
                .collect(),
        );
        palette
            .add_property(ROUGHNESS_FACTOR.to_owned(), roughness)
            .expect("the property names are distinct");
        color_axis.push(false);
        // Voxel Max glows in the voxel's own base color, so an emissive
        // material's color is its base color. The property appears only when
        // some material emits, and rides the color axis like `baseColorFactor`; the
        // emissive is then `emissiveFactor` times `emissiveStrength` per glTF, so
        // the color leads the strength that scales it.
        if materials.iter().any(|m| m.sic > 0.0) {
            let emissive_color = state.add_value_pool(VoxValuePool::srgb(
                colors
                    .iter()
                    .map(|color| {
                        let [r, g, b, _] =
                            <[f64; 4]>::from(TySrgbaU8::from(*color).into_format::<f64, f64>());
                        [r, g, b]
                    })
                    .collect(),
            ));
            palette
                .add_property(EMISSIVE_FACTOR.to_owned(), emissive_color)
                .expect("the property names are distinct");
            color_axis.push(true);
        }
        let emissive = float_pool(state, materials.iter().map(|m| m.sic).collect());
        palette
            .add_property(EMISSIVE_STRENGTH.to_owned(), emissive)
            .expect("the property names are distinct");
        color_axis.push(false);

        // Dispersion properties appear only when some material carries an `md`
        // block. A material without one takes the glTF default ior and zero
        // transmission and absorption; its absence rides in the ext.
        if materials.iter().any(|m| m.md.is_some()) {
            let default_ior = default_scalar(IOR).expect("ior has a glTF default");
            let ior = float_pool(
                state,
                materials
                    .iter()
                    .map(|m| m.md.as_ref().map_or(default_ior, |d| d.ior))
                    .collect(),
            );
            palette
                .add_property(IOR.to_owned(), ior)
                .expect("the property names are distinct");
            color_axis.push(false);
            let transmission = float_pool(state, dispersion(&materials, |d| d.transmission));
            palette
                .add_property(TRANSMISSION_FACTOR.to_owned(), transmission)
                .expect("the property names are distinct");
            color_axis.push(false);
            let absorption = float_pool(state, dispersion(&materials, |d| d.absorption));
            palette
                .add_property(ABSORPTION.to_owned(), absorption)
                .expect("the property names are distinct");
            color_axis.push(false);
        }

        let shadows = state.add_value_pool(VoxValuePool::boolean(
            materials.iter().map(|m| m.sh).collect(),
        ));
        palette
            .add_property(SHADOWS.to_owned(), shadows)
            .expect("the property names are distinct");
        color_axis.push(false);
    }

    // One material per distinct combination, ordered by color cell then
    // material byte, so the folded rows are canonical rather than voxel-scan
    // order and the round-trip is stable. The color index is 1-based in Voxel
    // Max, so a color-axis property takes `color_idx - 1`; the material byte
    // is 0-based, so every material-axis property takes it directly.
    let mut keys: Vec<(u8, u8)> = voxels
        .iter()
        .map(|voxel| combo_key(voxel, has_materials))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let mut combos: HashMap<(u8, u8), U32Id<BVoxMaterial>> = HashMap::new();
    for key in keys {
        let color = u32::from(key.0).saturating_sub(1);
        let material = u32::from(key.1);
        let value_ids = color_axis
            .iter()
            .map(|&is_color| U32Id::from_u32(if is_color { color } else { material }))
            .collect();
        let id = palette
            .add_material(value_ids)
            .expect("one value id per property");
        combos.insert(key, id);
    }

    let palette = state.add_palette(palette);
    let provenance = VoxelMaxPalette {
        name,
        materials: materials.iter().map(voxel_max_material).collect(),
    };
    FoldedPalette {
        palette,
        provenance,
        combos,
        has_materials,
    }
}

/// The color-and-material combination key for a voxel: its color index and, when
/// the object has materials, its material index, else zero.
fn combo_key(voxel: &VMaxVoxel, has_materials: bool) -> (u8, u8) {
    (
        voxel.color_idx,
        if has_materials { voxel.material_idx } else { 0 },
    )
}

/// The 0-based RGBA color table for an object. The `palette*.png` pixels when
/// present (its trailing transparent terminator dropped), else the material
/// sidecar's packed `colors` table, and finally a uniform placeholder so color
/// indices are still preserved.
fn color_cells(serde: &VMaxFile, object: &VMaxObject) -> Vec<[u8; 4]> {
    if let Some(png) = serde.palette_png_files.get(&object.palette) {
        return png.0.iter().take(COLOR_CELLS).copied().collect();
    }
    if let Some(stem) = object.palette.strip_suffix(".png") {
        let sidecar = format!("{stem}.settings.vmaxpsb");
        if let Some(palette) = serde.palette_settings_files.get(&sidecar)
            && !palette.colors.is_empty()
        {
            // The sidecar stores colors packed (4 bytes per cell); unpack them.
            return palette
                .colors
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect();
        }
    }
    (0..COLOR_CELLS).map(|_| PLACEHOLDER_COLOR).collect()
}

/// An object's material-palette display name and its exact material list from
/// the settings sidecar, or empty when it has no sidecar or no materials.
fn material_list(serde: &VMaxFile, object: &VMaxObject) -> (String, Vec<VMaxMaterial>) {
    let Some(stem) = object.palette.strip_suffix(".png") else {
        return (String::new(), Vec::new());
    };
    let sidecar = format!("{stem}.settings.vmaxpsb");
    match serde.palette_settings_files.get(&sidecar) {
        Some(settings) => (settings.name.clone(), settings.materials.clone()),
        None => (String::new(), Vec::new()),
    }
}

/// An unbounded float pool over `values`, defaulting a non-finite coefficient to
/// zero so the pool validates; the exact value rides in the ext.
fn float_pool(state: &mut VoxMain, values: Vec<f64>) -> U32Id<BVoxValuePool> {
    let values = values
        .into_iter()
        .map(|v| if v.is_finite() { v } else { 0.0 })
        .collect();
    state.add_value_pool(VoxValuePool::float(VoxBound::None, VoxBound::None, values))
}

/// Each material's dispersion field `read`, or zero where dispersion is absent.
fn dispersion(
    materials: &[VMaxMaterial],
    read: impl Fn(&VMaxMaterialDispersion) -> f64,
) -> Vec<f64> {
    materials
        .iter()
        .map(|m| m.md.as_ref().map_or(0.0, &read))
        .collect()
}

/// The exact ext copy of a Voxel Max material.
fn voxel_max_material(material: &VMaxMaterial) -> VoxelMaxMaterial {
    VoxelMaxMaterial {
        metallic: material.mc,
        roughness: material.rc,
        emissive: material.sic,
        shadows: material.sh,
        transmission_color: material.tc,
        dispersion: material.md.as_ref().map(|d| VoxelMaxMaterialDispersion {
            absorption: d.absorption,
            ior: d.ior,
            transmission: d.transmission,
        }),
    }
}

/// Builds the `voxel-max` ext payload: the scene-level state, the per-node
/// provenance aligned with the hierarchy nodes, the per-palette provenance, and
/// the per-object editor states.
fn voxel_max_ext(
    scene: &VMaxSceneJsonFile,
    palettes: Vec<Option<VoxelMaxPalette>>,
    object_states: &[Option<VoxelMaxObjectState>],
) -> VoxelMaxExt {
    let mut scene_block = scene.clone();
    scene_block.groups = Vec::new();
    scene_block.objects = Vec::new();

    // Aligned with the hierarchy nodes: groups first, then objects.
    let mut hierarchy_nodes: Vec<VoxelMaxNode> = scene.groups.iter().map(node_from_group).collect();
    hierarchy_nodes.extend(scene.objects.iter().map(node_from_object));

    VoxelMaxExt {
        scene: scene_block,
        hierarchy_nodes,
        palettes,
        object_states: object_states.to_vec(),
    }
}

/// The per-node provenance for a scene object. The content box is not kept; it
/// is derived on write from the object's native tight bounds.
fn node_from_object(object: &VMaxObject) -> VoxelMaxNode {
    VoxelMaxNode {
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
fn node_from_group(group: &VMaxGroup) -> VoxelMaxNode {
    VoxelMaxNode {
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

/// The minimum `[x, y, z]` corner over `voxels`, or `None` when empty.
fn min_corner(voxels: &[VMaxVoxel]) -> Option<[i32; 3]> {
    voxels
        .iter()
        .fold(None, |acc, v| {
            let position = TyVector3I32::from_array(v.position);
            let acc = acc.unwrap_or(position);
            Some(acc.min(position))
        })
        .map(|corner| corner.to_array())
}

/// The `[X, Y, Z]` bounds: the per-axis extent of `voxels` relative to
/// `box_min`.
fn object_bounds(voxels: &[VMaxVoxel], box_min: [i32; 3]) -> [u32; 3] {
    let mut bounds = [1u32; 3];
    for v in voxels {
        bounds[0] = bounds[0].max((v.position[0] - box_min[0] + 1) as u32);
        bounds[1] = bounds[1].max((v.position[1] - box_min[1] + 1) as u32);
        bounds[2] = bounds[2].max((v.position[2] - box_min[2] + 1) as u32);
    }
    bounds
}

/// Identifies scene objects that place the same geometry more than once. Voxel
/// Max instances a model by reusing a `contents*.vmaxb` and its palette across
/// objects, so objects sharing the `data`/`palette` filenames and the same
/// authored box decode to one identical object.
type InstanceKey = (String, String, [i32; 3], [u32; 3]);

/// The [`InstanceKey`] for an object, or `None` when it cannot be instanced.
fn instance_key(object: &VMaxObject) -> Option<InstanceKey> {
    if object.data.is_empty() {
        return None;
    }
    let (box_min, size) = authored_box(object)?;
    Some((object.data.clone(), object.palette.clone(), box_min, size))
}

/// The object's authored build volume (`tools.vp`) from its contents file, the
/// size the author was working in. `None` when the object has no contents or no
/// partition recorded.
fn view_box<'a>(serde: &'a VMaxFile, object: &VMaxObject) -> Option<&'a VMaxViewBox> {
    serde
        .contents_files
        .get(&object.data)?
        .tools
        .as_ref()?
        .vp
        .as_ref()
}

/// The integer grid `origin`: the min corner offset from the placing node in
/// the node's local voxel frame. `round(box_min - center)` so the node
/// transform's position lands on the content center (the pivot); any odd-extent
/// half-voxel remainder is absorbed by that position, keeping rendering exact.
fn pivot_origin(box_min: [i32; 3], center: [f64; 3]) -> [i32; 3] {
    (TyVector3I32::from_array(box_min).as_dvec3() - TyVector3F64::from_array(center))
        .round()
        .as_ivec3()
        .to_array()
}

/// The object's build volume (the author's `tools.vp`) as `(bounds, origin)` in
/// the node's local voxel frame, which contains the runtime grid. The `origin`
/// is the build volume's min corner offset from the node, `vp.min - box_min +
/// origin`; an object with no build volume takes a zero-margin volume equal to
/// its runtime grid.
fn edit_grid(
    view_box: Option<&VMaxViewBox>,
    box_min: [i32; 3],
    origin: [i32; 3],
    bounds: [u32; 3],
) -> ([u32; 3], [i32; 3]) {
    match view_box {
        Some(vp) => (
            [
                (vp.max[0] - vp.min[0] + 1).max(0) as u32,
                (vp.max[1] - vp.min[1] + 1).max(0) as u32,
                (vp.max[2] - vp.min[2] + 1).max(0) as u32,
            ],
            [
                vp.min[0] as i32 - box_min[0] + origin[0],
                vp.min[1] as i32 - box_min[1] + origin[1],
                vp.min[2] as i32 - box_min[2] + origin[2],
            ],
        ),
        None => (bounds, origin),
    }
}

/// The re-basing origin `round(center + bounds_min)` and `[X, Y, Z]` size from
/// an object's authored Voxel Max bounds, or `None` when it has none.
fn authored_box(object: &VMaxObject) -> Option<([i32; 3], [u32; 3])> {
    let (min, max) = (object.bounds_min?, object.bounds_max?);
    let box_min = (TyVector3F64::from_array(object.center) + TyVector3F64::from_array(min))
        .round()
        .as_ivec3()
        .to_array();
    let size = [
        (max[0] - min[0]).round().max(0.0) as u32,
        (max[1] - min[1]).round().max(0.0) as u32,
        (max[2] - min[2]).round().max(0.0) as u32,
    ];
    Some((box_min, size))
}

/// Decodes a stored `[x, y, z, angle]` axis-angle rotation into a quaternion.
fn axis_angle(rotation: [f64; 4]) -> TyQuaternionF64 {
    let [x, y, z, angle] = rotation;
    let axis = TyVector3F64::new(x, y, z);
    // An unrotated object stores [0, 0, 0, 0]; a zero axis has no direction to
    // normalize and from_axis_angle needs a unit axis.
    if axis.length() < ZERO_LENGTH_TOLERANCE {
        return TyQuaternionF64::IDENTITY;
    }
    TyQuaternionF64::from_axis_angle(axis.normalize(), angle)
}

/// The node transform that places an object so rotating the node pivots its
/// grid about the content center. Voxel Max renders a voxel at `t_p + center +
/// R*S*(voxel - center)`, and a voxel is `box_min + local`, which sits at
/// node-local `origin + local`, so the node position is `t_p + center +
/// R*S*(box_min - center - origin)`. The bracket is the sub-voxel remainder
/// `box_min - center - origin`, so the position lands on the pivot and
/// rendering stays exact for any integer `origin`.
fn object_transform(object: &VMaxObject, box_min: [i32; 3], origin: [i32; 3]) -> TyTransformF64 {
    let rotation = axis_angle(object.rotation);
    let scale = TyVector3F64::from_array(object.scale);
    let center = TyVector3F64::from_array(object.center);
    let box_min = TyVector3I32::from_array(box_min).as_dvec3();
    let origin = TyVector3I32::from_array(origin).as_dvec3();

    // t_p + center + R*S*(box_min - center - origin); the bracket is the
    // sub-voxel remainder.
    let offset = (box_min - center - origin) * scale;
    let position = TyVector3F64::from_array(object.position) + center + rotation * offset;

    TyTransformF64::new(position, rotation, scale)
}

/// The transform for a scene group, placed directly at its authored position.
fn group_transform(group: &VMaxGroup) -> TyTransformF64 {
    TyTransformF64::new(
        TyVector3F64::from_array(group.position),
        axis_angle(group.rotation),
        TyVector3F64::from_array(group.scale),
    )
}

/// Builds the voxcore hierarchy: one node per group then one per object, the
/// latter placing its geometry. `object_refs[i]` is the object that scene
/// object `i` places, so instances share a `child_objects` id. Returns the
/// nodes in id order and the root ids.
fn build_hierarchy(
    scene: &VMaxSceneJsonFile,
    object_transforms: &[TyTransformF64],
    object_refs: &[usize],
) -> (Vec<VoxHierarchyNode>, Vec<U32Id<BVoxHierarchyNode>>) {
    let mut nodes: Vec<VoxHierarchyNode> = Vec::new();
    let mut node_of_id: HashMap<&str, usize> = HashMap::new();
    let mut parents: Vec<Option<&str>> = Vec::new();

    for group in &scene.groups {
        node_of_id.insert(&group.id, nodes.len());
        parents.push(group.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: group.name.clone(),
            child_nodes: Vec::new(),
            child_objects: Vec::new(),
            transform: group_transform(group),
        });
    }
    for (index, object) in scene.objects.iter().enumerate() {
        node_of_id.insert(&object.id, nodes.len());
        parents.push(object.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: object.name.clone(),
            child_nodes: Vec::new(),
            child_objects: vec![U32Id::<BVoxObject>::from_u32(object_refs[index] as u32)],
            transform: object_transforms[index],
        });
    }

    let mut roots = Vec::new();
    for (node, parent) in parents.iter().enumerate() {
        match parent.and_then(|pid| node_of_id.get(pid)) {
            Some(&parent_node) => nodes[parent_node]
                .child_nodes
                .push(U32Id::from_u32(node as u32)),
            None => roots.push(U32Id::from_u32(node as u32)),
        }
    }

    (nodes, roots)
}

/// Invalid-data error from a message.
fn invalid(message: String) -> Error {
    Error::Invalid(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use vmax::{VMaxTools, VMaxViewBox};
    use vmax_codec::encode_vmax_snapshots;

    /// An empty object (zero voxels) with no authored content box
    /// (`e_mi`/`e_ma` absent) but a `tools.vp` build volume away from the
    /// origin. Voxel Max opens such a file, so the loader must too: seating
    /// `box_min` at `vp.min` keeps the edit grid containing the runtime grid.
    /// Previously `box_min` fell back to `[0, 0, 0]`, the edit grid was offset
    /// off the runtime point, and the containment validator rejected the load.
    fn empty_object_with_view_box_only() -> VMaxFile {
        let object = VMaxObject {
            name: "empty".to_owned(),
            data: "contents1.vmaxb".to_owned(),
            palette: String::new(),
            history: String::new(),
            id: "o".to_owned(),
            parent_id: None,
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: None,
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [128.0, 128.0, 16.0],
            bounds_min: None,
            bounds_max: None,
        };
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[]),
            uuid: "u".to_owned(),
            v: 4,
            tools: Some(VMaxTools {
                vp: Some(VMaxViewBox {
                    min: [112, 112, 0],
                    max: [143, 143, 31],
                }),
                ..Default::default()
            }),
            brush: None,
            cam: None,
            pal: None,
        };
        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents1.vmaxb".to_owned(), contents);
        VMaxFile {
            scene_json_file: VMaxSceneJsonFile {
                v: 4,
                objects: vec![object],
                ..Default::default()
            },
            contents_files,
            palette_settings_files: BTreeMap::new(),
            palette_png_files: BTreeMap::new(),
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
    fn empty_object_without_content_box_loads_from_its_view_box() {
        let state = from_vmax_file(&empty_object_with_view_box_only())
            .expect("an empty object with only a build volume must load");
        let id = U32Id::<BVoxObject>::from_u32(0);
        let object = state.object(id).expect("the one object");
        // The object's grid is the build volume (the 32^3 `vp`); it has no live
        // voxels, so its derived runtime extent is empty.
        assert_eq!(object.bounds(), TyVector3U32::new(32, 32, 32));
        assert_eq!(object.live_extent(), None);
    }
}
