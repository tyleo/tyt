use crate::{
    ABSORPTION, Error, ObjectPlacement, Placement, Result, SHADOWS, SYNTH_CAMERA,
    SceneCameraSource, VMaxColorFormat, VMaxExtMaterial, VMaxExtNode, VMaxExtObjectState,
    VMaxExtPalette, VMaxVoxMain, VMaxWriteOptions, axis_angle, ext_placements,
    pbr_factor_to_vm_coefficient, place_object, tighten,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use ty_math::{
    TyBoundsF64, TyLinSrgbaF64, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3F64,
    ZERO_LENGTH_TOLERANCE,
};
use vmax::{
    VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion, VMaxObject,
    VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneJsonFile,
    snapshots::{VMaxVoxel, encode_vmax_snapshots},
};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty, BVoxValuePoolValue,
    VoxExt, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool, VoxValuePoolValueRef,
    color::value_pool_color,
    material::{
        BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, IOR, METALLIC, ROUGHNESS, TRANSMISSION,
        default_scalar,
    },
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

/// The `pal` an object with no color palette borrows. An empty reference makes
/// Voxel Max read the package directory as a file and abort, so a colorless
/// object shares the first color palette's name and writes no file of its own.
const FALLBACK_PALETTE: &str = "palette1.png";

/// How far a node's rotation may drift from its preserved axis-angle before
/// the writer encodes the live rotation instead.
const ROTATION_TOLERANCE: f64 = 1e-9;

/// Writes a [`VMaxVoxMain`] to a Voxel Max document, the inverse of
/// [`from_vmax_file`](crate::from_vmax_file). A loaded document writes back
/// exactly through its ext. A state
/// [`to_vmax_vox_main`](crate::to_vmax_vox_main) gave its ext writes as a
/// document synthesized from the scene. The ext supplies each node's,
/// palette's, and object's provenance and the scene-level state. The scene
/// supplies the rest: names, transforms, parents, and content boxes. Nodes
/// write in listing order, children before parents when the listing has them
/// so, as Voxel Max's documents do. Errors when an entity has no ext entry.
pub fn to_vmax_file(main: &VMaxVoxMain, options: &VMaxWriteOptions) -> Result<VMaxFile> {
    let placements = ext_placements(main)?;

    let mut objects: Vec<VMaxObject> = Vec::new();
    let mut groups: Vec<VMaxGroup> = Vec::new();
    // Color palette id -> the `pal` filename written for it.
    let mut palette_files: BTreeMap<U32Id<BVoxPalette>, String> = BTreeMap::new();
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
        let rotation = node_rotation(ext_node, node);

        if node.child_object_ids.is_empty() {
            let (center, half) = subtree_box_local(main, placement.node_id, &mut box_memo);
            groups.push(group_from_node(placement, rotation, center, half));
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
            let folded = folded_ref(main, object);
            let plan = match folded.as_ref() {
                Some(folded) => {
                    let provenance = ext_entry(
                        main.ext().palettes.get(&folded.palette_id),
                        "palette",
                        folded.palette_id.to_u32(),
                    )?;
                    material_plan(main, folded, provenance)?
                }
                None => MaterialPlan::default(),
            };
            let suffix = suffix(object_id);
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
            // volume, the scene placement from the node transform.
            let (tight, object_placement) = place_object(object);
            let object_state = ext_entry(
                main.ext().object_states.get(&object_id),
                "object",
                object_id.to_u32(),
            )?;

            // Instances share one contents file: rebuild it once.
            let data = match contents_by_object.get(&object_id) {
                Some(data) => data.clone(),
                None => {
                    let voxels = reconstruct_voxels(
                        main,
                        &tight,
                        folded.as_ref(),
                        &plan,
                        object_placement.box_min,
                    )?;
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

            let pal = build_palette(
                main,
                folded.as_ref(),
                &plan,
                &mut palette_files,
                &mut palette_settings_files,
                &mut palette_png_files,
                options.color_format,
            )?;

            objects.push(object_from_node(
                node,
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

/// The entry for an entity, or the error for an ext out of step with the
/// scene.
fn ext_entry<'a, T>(entry: Option<&'a T>, what: &str, id: u32) -> Result<&'a T> {
    entry.ok_or_else(|| Error::invalid(format!("vmax ext holds no entry for {what} {id}")))
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

/// Applies the scene camera choice to the rebuilt scene. `Ext` leaves the
/// camera the ext supplied.
fn apply_scene_camera(scene: &mut VMaxSceneJsonFile, scene_camera: SceneCameraSource) {
    match scene_camera {
        SceneCameraSource::Ext => {}
        SceneCameraSource::Empty => scene.cam = Some(SYNTH_CAMERA),
        SceneCameraSource::Camera(camera) => scene.cam = Some(camera),
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

/// The folded palette an object references: the palette id, the optional
/// `baseColor` property, and the material properties in
/// order.
struct FoldedRef {
    palette_id: U32Id<BVoxPalette>,
    color_property_id: Option<U32Id<BVoxProperty>>,
    material_property_ids: Vec<(String, U32Id<BVoxProperty>)>,
}

/// The one folded palette an object references on its single layer, or `None`
/// when it references no layer.
fn folded_ref<T: VoxExt>(main: &VoxMain<T>, object: &VoxObject) -> Option<FoldedRef> {
    let (_, palette_id) = object.iter_layers().next()?;
    let palette = main.palette(palette_id)?;
    let color_property_id = palette.property_id_by_name(BASE_COLOR);
    let material_property_ids = palette
        .iter_properties()
        .filter(|(property_id, _)| Some(*property_id) != color_property_id)
        .map(|(property_id, property)| (property.name.clone(), property_id))
        .collect();
    Some(FoldedRef {
        palette_id,
        color_property_id,
        material_property_ids,
    })
}

/// The material reconstruction for a folded palette: each folded material's
/// Voxel Max `material_idx`, the exact material list for the settings sidecar,
/// and the sidecar display name.
#[derive(Default)]
struct MaterialPlan {
    name: String,
    material_indices: BTreeMap<U32Id<BVoxMaterial>, u8>,
    materials: Vec<VMaxMaterial>,
}

/// Reconstructs the Voxel Max materials for a folded palette. A
/// Voxel-Max-origin state carries the exact list in its ext along with the
/// slot each folded material draws. A state loaded from another format has no
/// such list, so the materials are derived from the value pools, one per
/// distinct material signature.
fn material_plan<T: VoxExt>(
    main: &VoxMain<T>,
    folded: &FoldedRef,
    provenance: &VMaxExtPalette,
) -> Result<MaterialPlan> {
    let name = provenance.name.clone();
    let palette = main
        .palette(folded.palette_id)
        .expect("a referenced palette");

    // A color-only palette folds no materials; every material writes index 0.
    if folded.material_property_ids.is_empty() {
        let material_indices = palette
            .iter_materials()
            .map(|material_id| (material_id, 0))
            .collect();
        return Ok(MaterialPlan {
            name,
            material_indices,
            materials: Vec::new(),
        });
    }

    // The slots follow the palette's materials through the hooks. A missing
    // one means a malformed ext. A hand-edited slot can still exceed the list,
    // hence the check.
    if !provenance.materials.is_empty() {
        let mut material_indices = BTreeMap::new();
        for material_id in palette.iter_materials() {
            let Some(&slot) = provenance.slots.get(&material_id) else {
                return Err(Error::invalid(format!(
                    "vmax ext palette holds no material slot for material {}",
                    material_id.to_u32()
                )));
            };
            if usize::from(slot) >= MATERIAL_SLOTS {
                return Err(Error::invalid(format!(
                    "a voxel references material {slot}, but a Voxel Max palette holds only \
                     {MATERIAL_SLOTS} material slots"
                )));
            }
            material_indices.insert(material_id, slot);
        }
        let materials = provenance
            .materials
            .iter()
            .enumerate()
            .map(|(slot, material)| vmax_material(slot, material))
            .collect();
        return Ok(MaterialPlan {
            name,
            material_indices,
            materials,
        });
    }

    derive_materials(main, folded, palette, name)
}

/// Derives a Voxel Max material per distinct signature of the material
/// properties, for a state that carries no exact material list. The signature
/// index is the `material_idx`, and each material reads its coefficients from
/// the value pools. Errors when the distinct materials exceed
/// [`MATERIAL_SLOTS`], since a Voxel Max palette holds only that many, so a
/// cross-format source with too many materials cannot be represented rather
/// than silently wrapping.
fn derive_materials<T: VoxExt>(
    main: &VoxMain<T>,
    folded: &FoldedRef,
    palette: &VoxPalette,
    name: String,
) -> Result<MaterialPlan> {
    // The linear luminance of a material's base color, the reference Voxel Max
    // glows against, or `None` when the palette carries no base color.
    let base_luminance = |material_id| -> Option<f64> {
        let color_property_id = folded.color_property_id?;
        let value_id = palette.value_id(material_id, color_property_id)?;
        let value_pool = property_value_pool(main, folded.palette_id, color_property_id)?;
        let [r, g, b, _] = value_pool_color(value_pool, value_id)?;
        let linear: TyLinSrgbaF64 = TySrgbaU8::from([r, g, b, 255])
            .into_format::<f64, f64>()
            .into_linear();
        Some(0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue)
    };

    let mut signatures: Vec<Vec<U32Id<BVoxValuePoolValue>>> = Vec::new();
    // Parallel to `signatures`: the base luminance of the first material to
    // claim each slot, the reference its emissive is read against.
    let mut base_luminances: Vec<Option<f64>> = Vec::new();
    let mut index_of: HashMap<Vec<U32Id<BVoxValuePoolValue>>, u8> = HashMap::new();
    let mut material_indices = BTreeMap::new();
    for material_id in palette.iter_materials() {
        let signature: Vec<U32Id<BVoxValuePoolValue>> = folded
            .material_property_ids
            .iter()
            .map(|(_, property_id)| {
                palette
                    .value_id(material_id, *property_id)
                    .unwrap_or(U32Id::from_u32(0))
            })
            .collect();
        let material_index = match index_of.get(&signature) {
            Some(&material_index) => material_index,
            None => {
                if signatures.len() >= MATERIAL_SLOTS {
                    return Err(Error::invalid(format!(
                        "an object needs more than {MATERIAL_SLOTS} materials, but a Voxel Max \
                         palette holds only that many material slots"
                    )));
                }
                let material_index = signatures.len() as u8;
                signatures.push(signature.clone());
                base_luminances.push(base_luminance(material_id));
                index_of.insert(signature, material_index);
                material_index
            }
        };
        material_indices.insert(material_id, material_index);
    }
    let materials = signatures
        .iter()
        .enumerate()
        .map(|(slot, signature)| {
            derived_material(
                main,
                folded.palette_id,
                &folded.material_property_ids,
                slot,
                signature,
                base_luminances[slot],
            )
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(MaterialPlan {
        name,
        material_indices,
        materials,
    })
}

/// One derived Voxel Max material. A coefficient reads from its property's
/// value pool at the signature's value id. Metalness and roughness map from the
/// 0 to 1 glTF factor to Voxel Max's 0.1 to 0.9 slider coefficient; see
/// [`pbr_factor_to_vm_coefficient`].
fn derived_material<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    properties: &[(String, U32Id<BVoxProperty>)],
    slot: usize,
    signature: &[U32Id<BVoxValuePoolValue>],
    base_luminance: Option<f64>,
) -> Result<VMaxMaterial> {
    // `None` when the palette binds no such property, so the caller takes the
    // vocabulary default. A property bound to a value pool holding no scalar
    // errors instead: the palette names a value this writer cannot read, and a
    // default would write a material the source never described.
    let scalar = |property: &str| -> Result<Option<f64>> {
        let Some(position) = properties.iter().position(|(name, _)| name == property) else {
            return Ok(None);
        };
        value_pool_scalar(
            main,
            palette_id,
            properties[position].1,
            signature[position],
        )
        .map(Some)
        .ok_or_else(|| {
            Error::invalid(format!(
                "`{property}` draws from a value pool holding no scalar"
            ))
        })
    };
    let flag = |property: &str| -> Option<bool> {
        let position = properties.iter().position(|(name, _)| name == property)?;
        value_pool_flag(
            main,
            palette_id,
            properties[position].1,
            signature[position],
        )
    };
    // The emissive color's linear luminance at this slot, or `None` when the
    // property is absent. Folded into Voxel Max's single self-illumination
    // coefficient, read relative to the base color the caller supplies.
    let emissive_luminance = || -> Option<f64> {
        let position = properties
            .iter()
            .position(|(name, _)| name == EMISSIVE_COLOR)?;
        let value_pool = property_value_pool(main, palette_id, properties[position].1)?;
        let [r, g, b, _] = value_pool_color(value_pool, signature[position])?;
        let linear: TyLinSrgbaF64 = TySrgbaU8::from([r, g, b, 255])
            .into_format::<f64, f64>()
            .into_linear();
        Some(0.2126 * linear.red + 0.7152 * linear.green + 0.0722 * linear.blue)
    };
    let carries = |property: &str| -> bool { properties.iter().any(|(name, _)| name == property) };
    let dispersed = carries(IOR) || carries(TRANSMISSION) || carries(ABSORPTION);
    Ok(VMaxMaterial {
        mi: (slot + 1).to_string(),
        // An unbound property renders at its vocabulary default, the same one
        // the glTF export writes, so the two exporters read one source model
        // the same way.
        mc: pbr_factor_to_vm_coefficient(unbound_scalar(scalar(METALLIC)?, METALLIC), METALLIC)?,
        rc: pbr_factor_to_vm_coefficient(unbound_scalar(scalar(ROUGHNESS)?, ROUGHNESS), ROUGHNESS)?,
        // Voxel Max glows in the voxel's base color at coefficient `sic`, so
        // the emissive folds to its luminance relative to the base color, times
        // `emissiveStrength`. This inverts the from-vmax split, which emits the
        // base color as the emissive color and `sic` as the strength. `sic` is
        // unbounded. Voxel Max's 0 to 100 slider is `sic` 0 to 20. A black
        // factor stays matte. A missing or black base color cannot normalize,
        // so the bare emissive luminance stands in. With no emissive color the
        // strength stands alone.
        sic: match emissive_luminance() {
            Some(emissive) => {
                let strength = unbound_scalar(scalar(EMISSIVE_STRENGTH)?, EMISSIVE_STRENGTH);
                match base_luminance {
                    Some(base) if base > 0.0 => strength * emissive / base,
                    _ => strength * emissive,
                }
            }
            None => scalar(EMISSIVE_STRENGTH)?.unwrap_or(0.0),
        },
        // Voxel Max casts shadows by default; a source without a shadows flag,
        // such as glTF, takes that default.
        sh: flag(SHADOWS).unwrap_or(true),
        tc: None,
        md: match dispersed {
            true => Some(VMaxMaterialDispersion {
                absorption: scalar(ABSORPTION)?.unwrap_or(0.0),
                ior: unbound_scalar(scalar(IOR)?, IOR),
                transmission: unbound_scalar(scalar(TRANSMISSION)?, TRANSMISSION),
            }),
            false => None,
        },
    })
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

/// `value` when the property is bound, else the glTF vocabulary default the
/// format gives `key`, so an unbound property writes what it renders as.
fn unbound_scalar(value: Option<f64>, key: &str) -> f64 {
    value.unwrap_or_else(|| {
        default_scalar(key).expect("a vocabulary scalar the vmax writer emits has a spec default")
    })
}

/// The `f64` value at `value_id` in a property's `float` value pool, or `None`.
fn value_pool_scalar<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    property_id: U32Id<BVoxProperty>,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<f64> {
    match property_value_pool(main, palette_id, property_id)?.value(value_id) {
        Some(VoxValuePoolValueRef::Float(number)) => Some(number),
        _ => None,
    }
}

/// The `bool` value at `value_id` in a property's `bool` value pool, or `None`.
fn value_pool_flag<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    property_id: U32Id<BVoxProperty>,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<bool> {
    match property_value_pool(main, palette_id, property_id)?.value(value_id) {
        Some(VoxValuePoolValueRef::Bool(flag)) => Some(flag),
        _ => None,
    }
}

/// The value pool a property draws from.
fn property_value_pool<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    property_id: U32Id<BVoxProperty>,
) -> Option<&VoxValuePool> {
    let value_pool_id = main
        .palette(palette_id)?
        .property(property_id)?
        .value_pool_id;
    main.value_pool(value_pool_id)
}

/// Re-bases the tight object's voxels to absolute model space, recovering each
/// one's `color_idx` from its material's `baseColor` value id and its
/// `material_idx` from the material plan. A colorless voxel takes index 1.
fn reconstruct_voxels<T: VoxExt>(
    main: &VoxMain<T>,
    object: &VoxObject,
    folded: Option<&FoldedRef>,
    plan: &MaterialPlan,
    box_min: [i32; 3],
) -> Result<Vec<VMaxVoxel>> {
    let layer_id = object.iter_layers().next().map(|(layer_id, _)| layer_id);
    object
        .iter_live()
        .map(|voxel_id| {
            let position = object
                .voxel_position(voxel_id)
                .expect("a live voxel is within the grid");
            let material_id =
                layer_id.and_then(|layer_id| object.voxel_material(voxel_id, layer_id));
            let color_index = match (folded, material_id) {
                (Some(folded), Some(material_id)) => voxel_color_index(main, folded, material_id)?,
                // A colorless voxel still needs a non-empty index, so it takes
                // 1, not the empty index 0.
                _ => 1,
            };
            // Voxel Max's material byte is 0-based: byte `n` selects
            // `materials[n]`. The per-color material map in the sidecar (`lc`)
            // drives what renders, but the byte is kept consistent with it.
            let material_index = material_id
                .and_then(|material_id| plan.material_indices.get(&material_id).copied())
                .unwrap_or(0);
            Ok(VMaxVoxel {
                position: [
                    position.x as i32 + box_min[0],
                    position.y as i32 + box_min[1],
                    position.z as i32 + box_min[2],
                ],
                material_idx: material_index,
                color_idx: color_index,
            })
        })
        .collect()
}

/// The 1-based Voxel Max color index a `material` samples through
/// `baseColor`. Errors when the color value id reaches
/// [`PALETTE_COLORS`], one past the last usable color, so a padded source
/// palette is fine as long as its referenced colors fit.
fn voxel_color_index<T: VoxExt>(
    main: &VoxMain<T>,
    folded: &FoldedRef,
    material_id: U32Id<BVoxMaterial>,
) -> Result<u8> {
    let Some(color_property_id) = folded.color_property_id else {
        return Ok(1);
    };
    let index = main
        .palette(folded.palette_id)
        .and_then(|palette| palette.value_id(material_id, color_property_id))
        .map_or(0, |value_id| value_id.to_u32());
    if index >= PALETTE_COLORS as u32 {
        return Err(Error::invalid(format!(
            "a voxel references color cell {index}, but a Voxel Max palette holds only \
             {PALETTE_COLORS} colors, so the source has more colors than fit"
        )));
    }
    Ok(index as u8 + 1)
}

/// Returns the `pal` filename for an object, building its color image and
/// material sidecar the first time the folded palette is seen. An object with
/// no color property borrows the default palette name and writes no file.
#[allow(clippy::too_many_arguments)]
fn build_palette<T: VoxExt>(
    main: &VoxMain<T>,
    folded: Option<&FoldedRef>,
    plan: &MaterialPlan,
    palette_files: &mut BTreeMap<U32Id<BVoxPalette>, String>,
    palette_settings_files: &mut BTreeMap<String, VMaxPaletteSettingsVmaxpsbFile>,
    palette_png_files: &mut BTreeMap<String, VMaxPalettePngFile>,
    vmax_color_format: VMaxColorFormat,
) -> Result<String> {
    // An object with no color property borrows the default palette name. An
    // empty reference is one Voxel Max cannot resolve. No file is written for
    // it.
    let Some((palette_id, color_property_id)) =
        folded.and_then(|folded| Some((folded.palette_id, folded.color_property_id?)))
    else {
        return Ok(FALLBACK_PALETTE.to_owned());
    };
    if let Some(name) = palette_files.get(&palette_id) {
        return Ok(name.clone());
    }
    // Voxel Max numbers palette files 1-based (`palette1`, `palette2`, ...); an
    // un-numbered `palette.png` breaks the plist color lookup when no image is
    // written.
    let stem = palette_files.len() + 1;
    let pal = format!("palette{stem}.png");

    let colors = color_palette_colors(main, palette_id, color_property_id)?;
    if matches!(
        vmax_color_format,
        VMaxColorFormat::Png | VMaxColorFormat::All
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
        !plan.materials.is_empty() || matches!(vmax_color_format, VMaxColorFormat::Plist);
    if write_sidecar {
        let sidecar = format!("palette{stem}.settings.vmaxpsb");
        // The plist `colors` table is the 255 colors with no terminator.
        let sidecar_colors = match vmax_color_format {
            VMaxColorFormat::Png => Vec::new(),
            VMaxColorFormat::Plist | VMaxColorFormat::All => colors,
        };
        // The per-color material map Voxel Max renders from: each used color
        // cell carries a bit for the material it draws.
        let (lc, indices, current) = color_material_map(main, palette_id, color_property_id, plan);
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
    Ok(pal)
}

/// The color property's value pool decoded to exactly [`PALETTE_COLORS`]
/// 0-based RGBA entries, padded with transparent entries or truncated to that
/// count. Colors past the budget are dropped; a voxel that would reference one
/// is rejected by [`reconstruct_voxels`].
///
/// Errors when the bound value pool holds no color because a transparent
/// stand-in would write a model Voxel Max renders as empty.
fn color_palette_colors<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    color_property_id: U32Id<BVoxProperty>,
) -> Result<Vec<[u8; 4]>> {
    let mut cells: Vec<[u8; 4]> = Vec::new();
    if let Some(value_pool) = property_value_pool(main, palette_id, color_property_id) {
        for (value_id, _) in value_pool.iter_values().take(PALETTE_COLORS) {
            let color = value_pool_color(value_pool, value_id).ok_or_else(|| {
                Error::invalid(format!(
                    "`{BASE_COLOR}` draws from a value pool holding no color"
                ))
            })?;
            cells.push(color);
        }
    }
    cells.resize(PALETTE_COLORS, [0, 0, 0, 0]);
    Ok(cells)
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
fn color_material_map<T: VoxExt>(
    main: &VoxMain<T>,
    palette_id: U32Id<BVoxPalette>,
    color_property_id: U32Id<BVoxProperty>,
    plan: &MaterialPlan,
) -> (Vec<u8>, Vec<i64>, i64) {
    let mut lc = vec![0u8; 256];
    if plan.materials.is_empty() {
        return (lc, Vec::new(), 0);
    }
    let mut cells: BTreeSet<u32> = BTreeSet::new();
    if let Some(palette_ref) = main.palette(palette_id) {
        for material_id in palette_ref.iter_materials() {
            let Some(cell) = palette_ref
                .value_id(material_id, color_property_id)
                .map(|value_id| value_id.to_u32())
            else {
                continue;
            };
            let Some(&byte) = plan.material_indices.get(&material_id) else {
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

#[allow(clippy::too_many_arguments)]
fn object_from_node(
    node: &VoxHierarchyNode,
    ext_node: &VMaxExtNode,
    parent_id: Option<String>,
    rotation: [f64; 4],
    placement: &ObjectPlacement,
    data: String,
    pal: String,
    suffix: &str,
) -> VMaxObject {
    VMaxObject {
        name: node.name.clone(),
        data,
        palette: pal,
        history: format!("history{suffix}.vmaxhb"),
        id: ext_node.id.clone(),
        parent_id,
        hidden: None,
        position: unbake_position(&node.transform, decode_axis_angle(rotation), placement),
        rotation,
        scale: node.transform.scale.to_array(),
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

/// The bounding box `(center, half)` of all geometry under `node_id`, in that
/// node's own local frame: the union of each child object's content box and
/// each child node's box mapped through the child's transform. Voxel Max stores
/// this per group as `e_c`/`e_mi`/`e_ma`; it is the union of the subtree, so it
/// is derived here rather than kept in the ext. Memoized by node id so a
/// subtree shared across parents is walked once. A node with no geometry
/// collapses to a zero box.
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
fn object_box_local<T: VoxExt>(
    main: &VoxMain<T>,
    object_id: U32Id<BVoxObject>,
) -> ([f64; 3], [f64; 3]) {
    let object = main.object(object_id).expect("a valid child object");
    let (tight, (edit_bounds, edit_origin)) = tighten(object);
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
fn group_from_node(
    placement: &Placement<'_>,
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
        position: node.transform.position.to_array(),
        rotation,
        scale: node.transform.scale.to_array(),
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

/// The axis-angle a node writes: the preserved spelling while it still
/// decodes to the node's rotation, which keeps a loaded document byte for
/// byte, or the live rotation encoded afresh once the node was rotated after
/// the load.
fn node_rotation(ext_node: &VMaxExtNode, node: &VoxHierarchyNode) -> [f64; 4] {
    let stored = decode_axis_angle(ext_node.rotation);
    let live = node.transform.rotation;
    if stored.abs_diff_eq(live, ROTATION_TOLERANCE) || stored.abs_diff_eq(-live, ROTATION_TOLERANCE)
    {
        return ext_node.rotation;
    }
    axis_angle(live)
}

/// Decodes a stored `[x, y, z, angle]` axis-angle like the read path so the
/// two stay inverses. A degenerate axis decodes to identity.
fn decode_axis_angle(rotation: [f64; 4]) -> TyQuaternionF64 {
    let [x, y, z, angle] = rotation;
    let axis = TyVector3F64::new(x, y, z);
    if axis.length() < ZERO_LENGTH_TOLERANCE {
        return TyQuaternionF64::IDENTITY;
    }
    TyQuaternionF64::from_axis_angle(axis.normalize(), angle)
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
fn suffix(object_id: U32Id<BVoxObject>) -> String {
    let index = object_id.to_u32();
    if index == 0 {
        String::new()
    } else {
        index.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        SceneCameraSource, VMaxExtNode, VMaxVoxMain, VMaxWriteOptions, from_vmax_file, to_vmax_file,
    };
    use branded_id::U32Id;
    use std::collections::{BTreeMap, BTreeSet, HashMap};
    use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxGroup, VMaxMaterial, VMaxMaterialDispersion,
        VMaxObject, VMaxPalettePngFile, VMaxPaletteSettingsVmaxpsbFile, VMaxSceneCamera,
        VMaxSceneJsonFile, VMaxViewBox,
        snapshots::{VMaxVoxel, decode_vmax_snapshots, encode_vmax_snapshots},
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePoolValue,
        VoxHierarchyNode, VoxObject, material::BASE_COLOR,
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
        // folds palette 1.
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let doomed_node_id = U32Id::<BVoxHierarchyNode>::from_u32(2);
        let doomed_object_id = U32Id::<BVoxObject>::from_u32(1);
        let doomed_palette_id = U32Id::<BVoxPalette>::from_u32(1);
        let mut group = main.hierarchy_node(group_id).unwrap().clone();
        group.child_node_ids.retain(|&id| id != doomed_node_id);
        main.set_hierarchy_node(group_id, group).unwrap();
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

    /// A node rotated after the load writes its live rotation, while an
    /// unrotated one keeps the preserved spelling.
    #[test]
    fn a_node_rotated_after_the_load_writes_its_live_rotation() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
        let mut group = main.hierarchy_node(group_id).unwrap().clone();
        group.transform.rotation = TyQuaternionF64::from_axis_angle(TyVector3F64::Z, 0.5);
        main.set_hierarchy_node(group_id, group).unwrap();

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

    /// A material retained to a palette with an exact material list takes the
    /// slot its material-axis value ids select, and the writer draws it. A
    /// retain after the value pools were pruned refuses, because the pools no
    /// longer index the list.
    #[test]
    fn a_retained_material_takes_the_slot_its_values_select() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // Color cell 0 in slot 1: every material-axis property takes value 1.
        let palette = main.palette(palette_id).unwrap();
        let value_ids: Vec<U32Id<BVoxValuePoolValue>> = palette
            .iter_properties()
            .map(|(_, property)| U32Id::from_u32(u32::from(property.name != BASE_COLOR)))
            .collect();
        let material_id = main.retain_material(palette_id, value_ids).unwrap();
        assert_eq!(main.ext().palettes[&palette_id].slots[&material_id], 1);

        main.prune_value_pools();
        let value_ids: Vec<U32Id<BVoxValuePoolValue>> = main
            .palette(palette_id)
            .unwrap()
            .iter_properties()
            .map(|_| U32Id::from_u32(0))
            .collect();
        assert!(main.retain_material(palette_id, value_ids).is_err());
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
    /// value pools, and compacts. The survivor's slot rides in the ext, so the
    /// document still draws its exact material even though its value ids were
    /// renumbered, and the reload records the one slot.
    #[test]
    fn a_reduction_keeps_the_survivors_material_slot_through_prune_and_gc() {
        let mut main = from_vmax_file(&sample()).unwrap();
        let palette_id = U32Id::<BVoxPalette>::from_u32(0);
        // The folded materials sort by color cell then slot: material 0 draws
        // cell 2 in slot 1 and material 1 draws cell 4 in slot 0. Keep material
        // 0, whose slot the pools stop recording once slot 0's values are
        // pruned away and its own renumber to 0.
        let doomed_id = U32Id::<BVoxMaterial>::from_u32(1);
        let survivor_id = U32Id::<BVoxMaterial>::from_u32(0);
        main.repaint_materials(palette_id, &HashMap::from([(doomed_id, survivor_id)]))
            .unwrap();
        main.release_material(palette_id, doomed_id).unwrap();
        main.prune_value_pools();
        main.gc().unwrap();
        assert_eq!(
            main.ext().palettes[&palette_id].slots,
            BTreeMap::from([(survivor_id, 1)])
        );

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();
        let voxels =
            decode_vmax_snapshots(&file.contents_files["contents.vmaxb"].snapshots).unwrap();
        assert!(voxels.iter().all(|voxel| voxel.material_idx == 1));
        let settings = &file.palette_settings_files["palette1.settings.vmaxpsb"];
        assert_eq!(settings.materials[1], material("2", 0.5, 0.25, 2.0, false));

        let reloaded = from_vmax_file(&file).unwrap();
        let palette = &reloaded.ext().palettes[&palette_id];
        assert_eq!(palette.slots, BTreeMap::from([(survivor_id, 1)]));
        assert_eq!(palette.materials.len(), 8);
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

    /// A material carrying dispersion, a transmission color (`tc`), and a
    /// non-finite coefficient round-trips exactly, alongside a plain material
    /// and the padded default slots. The value pools carry a finite-defaulted
    /// neutral copy, but the ext keeps the exact values, so the rebuilt
    /// document matches byte for byte. Coefficients are f32-exact, as Voxel Max
    /// stores them, and the non-finite `mc` is infinity, not NaN, so it
    /// compares equal.
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
