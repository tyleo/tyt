use crate::{
    ABSORPTION, Error, PALETTE_COLORS, Result, SHADOWS, VMaxExt, VMaxExtMaterial,
    VMaxExtMaterialDispersion, VMaxExtNode, VMaxExtObjectState, VMaxExtPalette, VMaxVoxMain,
    decode_axis_angle, synthesized_object_state, vm_coefficient_to_pbr_factor,
};
use branded_id::{U32Id, ext::IteratorExt};
use std::collections::{BTreeMap, HashMap};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use vmax::{
    VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxObject,
    VMaxSceneJsonFile, VMaxViewBox,
    snapshots::{VMaxVoxel, decode_vmax_snapshots},
};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePool, VoxHierarchyNode,
    VoxMain, VoxObject, VoxPalette, VoxValuePool,
    color::lin_srgba_f64_from_srgba_u8,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, ROUGHNESS, TRANSMISSION,
        default_scalar,
    },
};

/// The color used for every entry of a placeholder color table when an object's
/// colors are missing, so the color indices are still preserved.
const PLACEHOLDER_COLOR: [u8; 4] = [255, 255, 255, 255];

/// Loads a Voxel Max document into a [`VMaxVoxMain`], the inverse of
/// [`to_vmax_file`](crate::to_vmax_file()). Geometry, palettes, and hierarchy
/// become native voxcore entities. Voxel Max's axes run `+x` right, `+z` up,
/// and `-y` toward the viewer, so every object and node transform turns onto
/// voxcore's Y-up axes. The rest of the Voxel Max state becomes the ext. Voxel
/// snapshots are decoded to voxels on the fly and palette color tables unpacked
/// as needed. Color indices are 1-based in Voxel Max, so a voxel's color cell
/// is `color_idx - 1`. The material byte is 0-based and used directly.
///
/// Errors on malformed geometry or on a cross-reference the checked insertions
/// reject.
pub fn from_vmax_file(serde: &VMaxFile) -> Result<VMaxVoxMain> {
    let scene = &serde.scene_json_file;
    let mut main = VoxMain::default();

    // One palette per distinct object, by palette id.
    let mut palette_provenance: BTreeMap<U32Id<BVoxPalette>, VMaxExtPalette> = BTreeMap::new();

    // One voxcore object per distinct geometry; instances of one geometry
    // collapse to a single object placed by several nodes.
    let mut object_transforms: Vec<TyTransformF64> = Vec::new();
    let mut object_data: Vec<(U32Id<BVoxObject>, Option<String>)> = Vec::new();
    let mut object_ids: Vec<usize> = Vec::new();
    let mut instances: HashMap<InstanceKey, usize> = HashMap::new();
    for object in &scene.objects {
        let key = InstanceKey::of(object);
        if let Some(&existing) = key.as_ref().and_then(|key| instances.get(key)) {
            // An instance shares the geometry it re-places, so it re-derives
            // only the placing transform from its own content box and pivot.
            let box_min = authored_box(object).map_or([0, 0, 0], |(box_min, _)| box_min);
            let origin = pivot_origin(box_min, object.center);
            object_transforms.push(object_transform(object, box_min, origin).zup_to_yup());
            object_ids.push(existing);
            continue;
        }
        // The object and its placing transform turn together, so the node
        // keeps pivoting about the content center.
        let (vox_object, data, transform) =
            build_object(serde, object, &mut main, &mut palette_provenance)?;
        let object_id = main.retain_object(vox_object.zup_to_yup())?;
        object_data.push((object_id, data));
        object_transforms.push(transform.zup_to_yup());
        object_ids.push(object_id.to_u32() as usize);
        if let Some(key) = key {
            instances.insert(key, object_id.to_u32() as usize);
        }
    }

    // The scene nodes land as one batch: a group's children may sit after
    // it in the listing.
    let (nodes, roots) = build_hierarchy(scene, &object_transforms, &object_ids);
    let node_ids = main.retain_hierarchy_nodes(nodes)?;
    main.set_root_hierarchy_node_ids(roots)?;

    let ext = vmax_ext_from_file(serde, &main, &node_ids, palette_provenance, object_data);
    Ok(main.put_ext(ext))
}

/// Identifies scene objects that place the same geometry more than once. Voxel
/// Max instances a model by reusing a `contents*.vmaxb` and its palette across
/// objects, so objects sharing the `data`/`palette` filenames and the same
/// authored box decode to one identical object.
#[derive(Clone, Eq, Hash, PartialEq)]
struct InstanceKey {
    /// The `contents*.vmaxb` filename.
    data: String,

    /// The `palette*.png` filename.
    palette: String,

    /// The authored box's min corner.
    box_min: [i32; 3],

    /// The authored box's size.
    size: [u32; 3],
}

impl InstanceKey {
    /// The key for `object`, or `None` when it cannot be instanced.
    fn of(object: &VMaxObject) -> Option<Self> {
        if object.data.is_empty() {
            return None;
        }
        let (box_min, size) = authored_box(object)?;
        Some(InstanceKey {
            data: object.data.clone(),
            palette: object.palette.clone(),
            box_min,
            size,
        })
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

/// The node transform that places an object so rotating the node pivots its
/// grid about the content center. Voxel Max renders a voxel at `t_p + center +
/// R*S*(voxel - center)`, and a voxel is `box_min + local`, which sits at
/// node-local `origin + local`, so the node position is `t_p + center +
/// R*S*(box_min - center - origin)`. The bracket is the sub-voxel remainder
/// `box_min - center - origin`, so the position lands on the pivot and
/// rendering stays exact for any integer `origin`.
fn object_transform(object: &VMaxObject, box_min: [i32; 3], origin: [i32; 3]) -> TyTransformF64 {
    let rotation = decode_axis_angle(object.rotation);
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

/// Builds the voxcore object for one scene object, adding any palettes it
/// introduces to `state`. The object is built in its build volume (the author's
/// `tools.vp`), so its live voxels keep their authored positions inside it.
/// Returns the object, its backing `data` filename, and the transform of the
/// node that places it.
fn build_object(
    serde: &VMaxFile,
    object: &VMaxObject,
    main: &mut VoxMain<()>,
    palette_provenance: &mut BTreeMap<U32Id<BVoxPalette>, VMaxExtPalette>,
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
    .map_err(|_| {
        Error::invalid(format!(
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

    // One palette carries the color table and the material list: each voxel
    // samples a single material carrying both its color and its material
    // coefficients, one per distinct color-and-material combination the voxels
    // use.
    let palette = object_palette(serde, object, &voxels, main)?;
    palette_provenance.insert(palette.palette_id, palette.provenance);

    // Back-fill the layer with material 0; the live voxels overwrite theirs.
    vox_object.retain_layer(palette.palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for voxel in &voxels {
        // Shift the voxel from its model position into the build volume; an
        // out-of-grid result casts to a huge u32 and is rejected by `voxel_id`.
        let position = TyVector3U32::new(
            (voxel.position[0] - box_min[0] + offset[0]) as u32,
            (voxel.position[1] - box_min[1] + offset[1]) as u32,
            (voxel.position[2] - box_min[2] + offset[2]) as u32,
        );
        let voxel_id = vox_object.voxel_id(position).ok_or_else(|| {
            Error::invalid(format!(
                "object \"{}\" voxel ({}, {}, {}) lies outside its build volume",
                object.name, voxel.position[0], voxel.position[1], voxel.position[2]
            ))
        })?;

        let material_id = palette.combo_material_ids[&combo_key(voxel, palette.has_materials)];
        vox_object
            .retain_voxel(voxel_id, &[material_id])
            .map_err(|_| {
                Error::invalid(format!(
                    "object \"{}\" has a malformed voxel sample",
                    object.name
                ))
            })?;
    }

    Ok((vox_object, data, transform))
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

/// The palette built for one object and added to a [`VoxMain`], with the data
/// `build_object` needs to sample its voxels and record its ext provenance.
struct ObjectPalette {
    /// The palette id in the state.
    palette_id: U32Id<BVoxPalette>,

    /// The ext provenance carrying the name and exact material list.
    provenance: VMaxExtPalette,

    /// Each used color-and-material combination's material id.
    combo_material_ids: HashMap<(u8, u8), U32Id<BVoxMaterial>>,

    /// Whether the object carries materials.
    has_materials: bool,
}

/// Builds one palette for an object's live voxels, adding it and its value
/// pools to `main`. Voxel Max's color table and material list become one
/// palette: `baseColor` and `emissiveColor` ride the color axis, one value
/// per color cell, every material scalar rides the material axis, one value
/// per material slot, and one material per color-and-material combination
/// the voxels use gathers a value on each.
///
/// The color value pool is the object's full color table in order, so a
/// material's `baseColor` value-index is `color_idx - 1`. Each material
/// scalar value pool holds one value per Voxel Max material, in order; the
/// material byte is 0-based, so a voxel's value-index is its `material_idx`.
/// The exact material list rides in the ext provenance for a byte-exact
/// write-back.
fn object_palette(
    serde: &VMaxFile,
    object: &VMaxObject,
    voxels: &[VMaxVoxel],
    main: &mut VoxMain<()>,
) -> Result<ObjectPalette> {
    let colors = color_cells(serde, object);
    let (name, materials) = material_list(serde, object);
    let has_materials = !materials.is_empty();

    // `color_axis` records each property's axis in property order, so a
    // material gathers one value id per property.
    let mut palette = VoxPalette::default();
    let mut color_axis: Vec<bool> = Vec::new();
    let color_value_pool_id = main.retain_value_pool(VoxValuePool::vec_4_float(
        colors
            .iter()
            .map(|color| <[f64; 4]>::from(lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(*color))))
            .collect(),
    )?);
    palette
        .retain_property(
            BASE_COLOR.to_owned(),
            color_value_pool_id,
            U32Id::from_u32(0),
        )
        .expect("the property names are distinct");
    color_axis.push(true);

    // Metalness and roughness convert from Voxel Max's 0.1 to 0.9 slider
    // coefficient to the 0 to 1 glTF factor the value-pool name implies; see
    // [`vm_coefficient_to_pbr_factor`]. The remaining scalars are unbounded and
    // stay raw: `sic` is an unbounded emission strength, and `shadows` and
    // `absorption` have no glTF counterpart. The exact coefficients ride in the
    // ext for a byte-exact write-back.
    if has_materials {
        let metallic_value_pool_id = float_value_pool(
            main,
            materials
                .iter()
                .map(|m| vm_coefficient_to_pbr_factor(m.mc))
                .collect(),
        )?;
        palette
            .retain_property(
                METALLIC.to_owned(),
                metallic_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);
        let roughness_value_pool_id = float_value_pool(
            main,
            materials
                .iter()
                .map(|m| vm_coefficient_to_pbr_factor(m.rc))
                .collect(),
        )?;
        palette
            .retain_property(
                ROUGHNESS.to_owned(),
                roughness_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);
        // Voxel Max glows in the voxel's own base color, so an emissive
        // material's color is its base color. The property appears only when
        // some material emits, and rides the color axis like `baseColor`; the
        // emissive is then `emissiveFactor` times `emissiveStrength` per glTF,
        // so the color leads the strength that scales it.
        if materials.iter().any(|m| m.sic > 0.0) {
            let emissive_color_value_pool_id = main.retain_value_pool(VoxValuePool::vec_3_float(
                colors
                    .iter()
                    .map(|color| {
                        let linear = lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(*color));
                        [linear.red, linear.green, linear.blue]
                    })
                    .collect(),
            )?);
            palette
                .retain_property(
                    EMISSIVE_COLOR.to_owned(),
                    emissive_color_value_pool_id,
                    U32Id::from_u32(0),
                )
                .expect("the property names are distinct");
            color_axis.push(true);
        }
        let emissive_value_pool_id =
            float_value_pool(main, materials.iter().map(|m| m.sic).collect())?;
        palette
            .retain_property(
                EMISSIVE_STRENGTH.to_owned(),
                emissive_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);

        // Dispersion properties appear only when some material carries an `md`
        // block. A material without one takes the glTF default ior and zero
        // transmission and absorption; its absence rides in the ext.
        if materials.iter().any(|m| m.md.is_some()) {
            let default_ior = default_scalar(IOR).expect("ior has a glTF default");
            let ior_value_pool_id = float_value_pool(
                main,
                materials
                    .iter()
                    .map(|m| m.md.as_ref().map_or(default_ior, |d| d.ior))
                    .collect(),
            )?;
            palette
                .retain_property(IOR.to_owned(), ior_value_pool_id, U32Id::from_u32(0))
                .expect("the property names are distinct");
            color_axis.push(false);
            let transmission_value_pool_id =
                float_value_pool(main, dispersion(&materials, |d| d.transmission))?;
            palette
                .retain_property(
                    TRANSMISSION.to_owned(),
                    transmission_value_pool_id,
                    U32Id::from_u32(0),
                )
                .expect("the property names are distinct");
            color_axis.push(false);
            let absorption_value_pool_id =
                float_value_pool(main, dispersion(&materials, |d| d.absorption))?;
            palette
                .retain_property(
                    ABSORPTION.to_owned(),
                    absorption_value_pool_id,
                    U32Id::from_u32(0),
                )
                .expect("the property names are distinct");
            color_axis.push(false);
        }

        let shadows_value_pool_id = main.retain_value_pool(VoxValuePool::boolean(
            materials.iter().map(|m| m.sh).collect(),
        ));
        palette
            .retain_property(
                SHADOWS.to_owned(),
                shadows_value_pool_id,
                U32Id::from_u32(0),
            )
            .expect("the property names are distinct");
        color_axis.push(false);
    }

    // One material per distinct combination, ordered by color cell then
    // material byte, so the material rows are canonical rather than voxel-scan
    // order and the round-trip is stable. The color index is 1-based in Voxel
    // Max, so a color-axis property takes `color_idx - 1`; the material byte
    // is 0-based, so every material-axis property takes it directly.
    let mut keys: Vec<(u8, u8)> = voxels
        .iter()
        .map(|voxel| combo_key(voxel, has_materials))
        .collect();
    keys.sort_unstable();
    keys.dedup();
    let mut combo_material_ids: HashMap<(u8, u8), U32Id<BVoxMaterial>> = HashMap::new();
    for key in keys {
        let color_index = u32::from(key.0).saturating_sub(1);
        let material_index = u32::from(key.1);
        let value_ids = color_axis
            .iter()
            .map(|&is_color| {
                U32Id::from_u32(if is_color {
                    color_index
                } else {
                    material_index
                })
            })
            .collect();
        let material_id = palette
            .retain_material(value_ids)
            .expect("one value id per property");
        combo_material_ids.insert(key, material_id);
    }

    let palette_id = main.retain_palette(palette)?;
    let provenance = VMaxExtPalette {
        name,
        materials: materials.iter().map(vmax_ext_material).collect(),
    };
    Ok(ObjectPalette {
        palette_id,
        provenance,
        combo_material_ids,
        has_materials,
    })
}

/// The color-and-material combination key for a voxel: its color index and,
/// when the object has materials, its material index, else zero.
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
        return png.0.iter().take(PALETTE_COLORS).copied().collect();
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
    (0..PALETTE_COLORS).map(|_| PLACEHOLDER_COLOR).collect()
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

/// A float value pool over `values`. Errors when `values` is empty or holds a
/// NaN.
fn float_value_pool(main: &mut VoxMain<()>, values: Vec<f64>) -> Result<U32Id<BVoxValuePool>> {
    Ok(main.retain_value_pool(VoxValuePool::float(values)?))
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
fn vmax_ext_material(material: &VMaxMaterial) -> VMaxExtMaterial {
    VMaxExtMaterial {
        metallic: material.mc,
        roughness: material.rc,
        emissive: material.sic,
        shadows: material.sh,
        transmission_color: material.tc,
        dispersion: material.md.as_ref().map(|d| VMaxExtMaterialDispersion {
            absorption: d.absorption,
            ior: d.ior,
            transmission: d.transmission,
        }),
    }
}

/// Builds the voxcore hierarchy: one node per group then one per object, the
/// latter placing its geometry. `object_ids[i]` is the object that scene object
/// `i` places, so instances share a `child_objects` id. `object_transforms`
/// already sit on Y-up axes. Group transforms turn here. Returns the nodes in
/// id order and the root ids.
fn build_hierarchy(
    scene: &VMaxSceneJsonFile,
    object_transforms: &[TyTransformF64],
    object_ids: &[usize],
) -> (Vec<VoxHierarchyNode>, Vec<U32Id<BVoxHierarchyNode>>) {
    let mut nodes: Vec<VoxHierarchyNode> = Vec::new();
    let mut node_index_of_id: HashMap<&str, usize> = HashMap::new();
    let mut parents: Vec<Option<&str>> = Vec::new();

    for group in &scene.groups {
        node_index_of_id.insert(&group.id, nodes.len());
        parents.push(group.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: group.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids: Vec::new(),
            transform: group_transform(group).zup_to_yup(),
        });
    }
    for (index, object) in scene.objects.iter().enumerate() {
        node_index_of_id.insert(&object.id, nodes.len());
        parents.push(object.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: object.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![U32Id::<BVoxObject>::from_u32(object_ids[index] as u32)],
            transform: object_transforms[index],
        });
    }

    let mut roots = Vec::new();
    for (node_id, parent) in parents.iter().enumerate_ids() {
        match parent.and_then(|pid| node_index_of_id.get(pid)) {
            Some(&parent_node_index) => nodes[parent_node_index].child_node_ids.push(node_id),

            None => roots.push(node_id),
        }
    }

    (nodes, roots)
}

/// The transform for a scene group, placed directly at its authored position.
fn group_transform(group: &VMaxGroup) -> TyTransformF64 {
    TyTransformF64::new(
        TyVector3F64::from_array(group.position),
        decode_axis_angle(group.rotation),
        TyVector3F64::from_array(group.scale),
    )
}

/// The ext of a read of `serde` into `main`.
///
/// 1. `node_ids`: the hierarchy node ids in scene order, groups then objects
/// 2. `palettes`: each object palette's provenance by palette id
/// 3. `object_data`: each object's contents filename by object id, or `None`
///    for an object with no contents file, which takes a synthesized editor
///    state
fn vmax_ext_from_file(
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

#[cfg(test)]
mod tests {
    use crate::from_vmax_file;
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet};
    use ty_math::{TyVector3F64, TyVector3U32};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxObject, VMaxSceneJsonFile, VMaxTools, VMaxViewBox,
        snapshots::{VMaxVoxel, encode_vmax_snapshots},
    };
    use voxcore::{BVoxHierarchyNode, BVoxObject};

    /// An empty object (zero voxels) with no authored content box
    /// (`e_mi`/`e_ma` absent) but a `tools.vp` build volume away from the
    /// origin. Voxel Max opens such a file, so the loader must too: seating
    /// `box_min` at `vp.min` keeps the edit grid containing the runtime grid.
    fn empty_object_with_view_box_only() -> VMaxFile {
        one_object_file(
            [128.0, 128.0, 16.0],
            VMaxViewBox {
                min: [112, 112, 0],
                max: [143, 143, 31],
            },
            &[],
        )
    }

    /// A document with one object at the scene origin, given on Voxel Max's
    /// axes.
    fn one_object_file(center: [f64; 3], vp: VMaxViewBox, voxels: &[VMaxVoxel]) -> VMaxFile {
        let object = VMaxObject {
            name: "one".to_owned(),
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
            center,
            bounds_min: None,
            bounds_max: None,
        };
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(voxels),
            uuid: "u".to_owned(),
            v: 4,
            tools: Some(VMaxTools {
                vp: Some(vp),
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
        let main = from_vmax_file(&empty_object_with_view_box_only())
            .expect("an empty object with only a build volume must load");
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        let object = main.object(object_id).expect("the one object");
        // The object's grid is the build volume (the 32^3 `vp`); it has no live
        // voxels, so its derived runtime extent is empty.
        assert_eq!(object.bounds(), TyVector3U32::new(32, 32, 32));
        assert_eq!(object.live_extent(), None);
    }

    /// An object and its placing node turn onto Y-up axes: Voxel Max's `+z`
    /// becomes `+y` and its `+y` becomes `-z`.
    #[test]
    fn turns_objects_and_transforms_onto_y_up() {
        let voxel = |x: i32, y: i32, z: i32| VMaxVoxel {
            position: [x, y, z],
            material_idx: 0,
            color_idx: 1,
        };
        let file = one_object_file(
            [1.0, 1.5, 2.0],
            VMaxViewBox {
                min: [0, 0, 0],
                max: [1, 2, 3],
            },
            &[voxel(0, 0, 0), voxel(1, 2, 3)],
        );
        let main = from_vmax_file(&file).expect("a placed object loads");

        let object = main
            .object(U32Id::<BVoxObject>::from_u32(0))
            .expect("the one object");
        assert_eq!(object.bounds(), TyVector3U32::new(2, 4, 3));
        let live: BTreeSet<[u32; 3]> = object
            .iter_live()
            .map(|voxel_id| {
                object
                    .voxel_position(voxel_id)
                    .expect("a live cell")
                    .to_array()
            })
            .collect();
        assert_eq!(live, BTreeSet::from([[0, 0, 2], [1, 3, 0]]));

        // The node sits at the content center `[1, 1.5, 2]` plus the half
        // voxel the integer origin rounds past on `y`: `[1, 2, 2]` on Voxel
        // Max's axes, `[1, 2, -2]` turned.
        let node = main
            .hierarchy_node(U32Id::<BVoxHierarchyNode>::from_u32(0))
            .expect("the placing node");
        assert_eq!(node.transform.position, TyVector3F64::new(1.0, 2.0, -2.0));
    }
}
