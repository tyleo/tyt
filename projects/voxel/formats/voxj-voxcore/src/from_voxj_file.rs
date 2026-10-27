use crate::{
    Error, Result, VoxjVoxExt, VoxjVoxMain, vox_map_from_voxj_map, vox_value_from_voxj_value,
};
use branded_id::U32Id;
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValuePool,
};
use voxj::{
    DecodeBase64, VoxjFile, VoxjHierarchyNode, VoxjPalette, VoxjTransform, VoxjValuePool,
    objects::{VoxjDecodedObject, decode_voxj_object, voxj_palette_material_counts},
};

/// Loads a [`VoxjFile`] into a [`VoxjVoxMain`] carrying its `ext` block as it
/// was parsed, the inverse of [`to_voxj_file`](crate::to_voxj_file()). A
/// document with no block loads an empty ext. Each object's position and sample
/// blocks decode through `dependencies`. Entities take ids in listing order, so
/// each id equals its voxj array index and the cross-references carry over. The
/// nodes land as one batch because the wire permits a node to list a child that
/// appears later.
///
/// Errors if:
///
/// 1. a block is malformed
/// 2. object geometry is malformed
/// 3. a checked insertion rejects a cross-reference
/// 4. the `ext` block holds a non-finite number or a repeated key
pub fn from_voxj_file<D: DecodeBase64>(dependencies: &D, file: &VoxjFile) -> Result<VoxjVoxMain> {
    let voxj_main = &file.main;
    let mut main = VoxMain::default();

    // Build each value before adding it so a failed conversion leaves the state
    // untouched. Value pools land first, so palette properties resolve against
    // them.
    for value_pool in &voxj_main.runtime_state.value_pools {
        main.retain_value_pool(vox_value_pool_from_voxj_value_pool(value_pool)?);
    }

    // An insertion identifies the entity it rejected by the ids it holds, which
    // are internal to the palette or object; the listing index points back into
    // the document.
    for (index, palette) in voxj_main.runtime_state.palettes.iter().enumerate() {
        main.retain_palette(vox_palette_from_voxj_palette(palette)?)
            .map_err(|error| Error::invalid(format!("palette {index}: {error}")))?;
    }

    for (index, object) in voxj_main.runtime_state.objects.iter().enumerate() {
        let material_counts =
            voxj_palette_material_counts(&object.layers, &voxj_main.runtime_state.palettes)?;
        let decoded = decode_voxj_object(dependencies, object, &material_counts)?;
        // The build volume, present only when the document recorded margin
        // around the object's live voxels; otherwise the runtime grid is the
        // build volume.
        let edit = voxj_main
            .edit_state
            .as_ref()
            .and_then(|e| e.objects.get(index))
            .map(|edit| (edit.bounds, edit.origin));
        let vox_object = vox_object_from_voxj_decoded_object(&decoded, edit)?;
        main.retain_object(vox_object)
            .map_err(|error| Error::invalid(format!("object {index}: {error}")))?;
    }

    let nodes = voxj_main
        .runtime_state
        .nodes
        .iter()
        .map(vox_hierarchy_node_from_voxj_hierarchy_node)
        .collect::<Result<Vec<_>>>()?;
    main.retain_hierarchy_nodes(nodes)?;

    main.set_root_hierarchy_node_ids(
        voxj_main
            .runtime_state
            .root_nodes
            .iter()
            .map(|&index| wire_id(index, "root node"))
            .collect::<Result<_>>()?,
    )?;

    let slots = match &voxj_main.ext {
        Some(block) => vox_map_from_voxj_map(block)?.into_entries(),
        None => Vec::new(),
    };

    Ok(main.put_ext(VoxjVoxExt::new(slots)))
}

/// Converts a [`VoxjValuePool`] into a [`VoxValuePool`], kind by kind. Every
/// kind maps one to one, carrying its values across unchanged. `json` values
/// recurse through [`vox_value_from_voxj_value`].
///
/// Errors if a value is outside its kind's value domain.
fn vox_value_pool_from_voxj_value_pool(value_pool: &VoxjValuePool) -> Result<VoxValuePool> {
    Ok(match value_pool {
        VoxjValuePool::Bool(values) => VoxValuePool::boolean(values.clone()),

        VoxjValuePool::Float(values) => VoxValuePool::float(values.clone())?,

        VoxjValuePool::Int(values) => VoxValuePool::int(values.clone())?,

        VoxjValuePool::Json(values) => VoxValuePool::json(
            values
                .iter()
                .map(vox_value_from_voxj_value)
                .collect::<Result<_>>()?,
        ),

        VoxjValuePool::String(values) => VoxValuePool::string(values.clone()),

        VoxjValuePool::Vec2Float(values) => VoxValuePool::vec_2_float(values.clone())?,

        VoxjValuePool::Vec2Int(values) => VoxValuePool::vec_2_int(values.clone())?,

        VoxjValuePool::Vec3Float(values) => VoxValuePool::vec_3_float(values.clone())?,

        VoxjValuePool::Vec3Int(values) => VoxValuePool::vec_3_int(values.clone())?,

        VoxjValuePool::Vec4Float(values) => VoxValuePool::vec_4_float(values.clone())?,

        VoxjValuePool::Vec4Int(values) => VoxValuePool::vec_4_int(values.clone())?,
    })
}

/// Builds a [`VoxPalette`] from a [`VoxjPalette`], in listing order so each
/// property and material id equals its wire index.
///
/// Properties carry over as name plus value-pool reference, the wire
/// `valuePool` becoming a value-pool id. `materials` carries over one row per
/// material, a value-index per property.
///
/// Errors on a duplicate property name, a row whose length disagrees with the
/// properties, or a value-pool reference or value-index past the id space.
/// Value-pool-reference and value-id ranges are checked when the palette is
/// inserted by
/// [`VoxMain::retain_palette`](voxcore::VoxMain::retain_palette).
fn vox_palette_from_voxj_palette(palette: &VoxjPalette) -> Result<VoxPalette> {
    let mut out = VoxPalette::default();

    for property in &palette.properties {
        out.retain_property(
            property.name.clone(),
            wire_id(property.value_pool, "value pool")?,
        )
        .map_err(|_| {
            Error::Invalid(format!(
                "palette declares property \"{}\" more than once",
                property.name
            ))
        })?;
    }

    for (index, row) in palette.materials.iter().enumerate() {
        let value_ids = row
            .iter()
            .map(|&value_index| {
                // A wire index past the id space would wrap onto a real value
                // and bind the material to a value the file never named.
                u32::try_from(value_index)
                    .map(U32Id::from_u32)
                    .map_err(|_| {
                        Error::Invalid(format!(
                            "palette material {index} names value-index {value_index}, past the \
                             value-index space"
                        ))
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        out.retain_material(value_ids).map_err(|_| {
            Error::Invalid(format!(
                "palette material {index} has {} value-indices but {} properties",
                row.len(),
                palette.properties.len()
            ))
        })?;
    }

    Ok(out)
}

/// Builds a [`VoxObject`] from a [`VoxjDecodedObject`] and its optional build
/// volume.
///
/// The decoded object holds the tight runtime grid. When `edit` is `Some`, the
/// object is built in that build volume with each voxel shifted from the
/// runtime grid into it, recovering the margin the document recorded. When
/// `edit` is `None`, the build volume equals the tight grid.
///
/// Each `layers` entry becomes a layer over that palette. A decoded sample row
/// holds one material index per layer. Errors on an oversized grid, a layer
/// index past the palette-index space, a position outside the grid, or ragged
/// sample rows. Cross-references are checked on insert by
/// [`VoxMain::retain_object`](voxcore::VoxMain::retain_object).
fn vox_object_from_voxj_decoded_object(
    object: &VoxjDecodedObject,
    edit: Option<([u32; 3], [i32; 3])>,
) -> Result<VoxObject> {
    // The grid the voxcore object lives in (its build volume), its placing
    // origin, and the offset shifting a runtime-grid position into it.
    let (bounds, origin, offset) = match edit {
        Some((bounds, origin)) => (
            bounds,
            origin,
            [
                object.origin[0] - origin[0],
                object.origin[1] - origin[1],
                object.origin[2] - origin[2],
            ],
        ),

        None => (object.bounds, object.origin, [0, 0, 0]),
    };
    let [size_x, size_y, size_z] = bounds;

    let mut out =
        VoxObject::new(object.name.clone(), TyVector3U32::from_array(bounds)).map_err(|_| {
            Error::invalid(format!(
                "object \"{}\" grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                object.name,
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    out.set_origin(TyVector3I32::from_array(origin));

    for &palette_index in &object.layers {
        // A layer index past the id space would wrap onto a real palette.
        let Ok(index) = u32::try_from(palette_index) else {
            return Err(Error::invalid(format!(
                "object \"{}\" layer references palette {palette_index}, past the palette-index \
                 space",
                object.name
            )));
        };

        out.retain_layer(U32Id::<BVoxPalette>::from_u32(index))
            .expect("a new object has no live voxels");
    }

    if object.samples.len() != object.positions.len() {
        return Err(Error::invalid(format!(
            "object \"{}\" has {} sample rows but {} positions",
            object.name,
            object.samples.len(),
            object.positions.len()
        )));
    }

    for (&[x, y, z], row) in object.positions.iter().zip(&object.samples) {
        let shifted = [
            x as i64 + offset[0] as i64,
            y as i64 + offset[1] as i64,
            z as i64 + offset[2] as i64,
        ];
        let position = in_bounds(shifted, bounds).ok_or_else(|| {
            Error::invalid(format!(
                "object \"{}\" position [{x}, {y}, {z}] lies outside its grid [{size_x}, {size_y}, {size_z}]",
                object.name
            ))
        })?;
        // `in_bounds` already confirmed the position fits the grid.
        let voxel_id = out.voxel_id(position).expect("position is within bounds");

        if row.len() != object.layers.len() {
            return Err(Error::invalid(format!(
                "object \"{}\" sample row at [{x}, {y}, {z}] has {} values but references {} \
                 layers",
                object.name,
                row.len(),
                object.layers.len()
            )));
        }

        let material_ids: Vec<U32Id<BVoxMaterial>> =
            row.iter().copied().map(U32Id::from_u32).collect();

        out.retain_voxel(voxel_id, &material_ids)
            .expect("the row has one material per layer");
    }

    Ok(out)
}

/// The id that wire index `index` references. Errors when `index` is past the
/// id space. A cast would wrap such an index onto a real entry.
fn wire_id<TBrand>(index: usize, what: &str) -> Result<U32Id<TBrand>> {
    let Ok(id) = u32::try_from(index) else {
        return Err(Error::Invalid(format!(
            "{what} index {index} is past the id space"
        )));
    };

    Ok(U32Id::from_u32(id))
}

/// The `[x, y, z]` point as a grid position, or `None` if any axis is negative
/// or reaches `bounds`.
fn in_bounds(p: [i64; 3], bounds: [u32; 3]) -> Option<TyVector3U32> {
    let inside = (0..3).all(|a| p[a] >= 0 && p[a] < bounds[a] as i64);
    inside.then(|| TyVector3U32::new(p[0] as u32, p[1] as u32, p[2] as u32))
}

/// Builds a [`VoxHierarchyNode`] from a [`VoxjHierarchyNode`], mapping child
/// indices to ids and the transform to its [`ty_math`] form. Child ids are
/// checked on insert by
/// [`VoxMain::retain_hierarchy_nodes`](voxcore::VoxMain::retain_hierarchy_nodes),
/// not here.
///
/// Errors on a child index past the id space or a degenerate transform. A
/// degenerate transform has a non-finite position, a non-finite or zero scale,
/// or a non-finite or zero rotation.
fn vox_hierarchy_node_from_voxj_hierarchy_node(
    node: &VoxjHierarchyNode,
) -> Result<VoxHierarchyNode> {
    Ok(VoxHierarchyNode {
        name: node.name.clone(),
        child_node_ids: node
            .child_nodes
            .iter()
            .map(|&index| wire_id(index, "child node"))
            .collect::<Result<_>>()?,
        child_object_ids: node
            .child_objects
            .iter()
            .map(|&index| wire_id(index, "child object"))
            .collect::<Result<_>>()?,
        transform: vox_transform_from_voxj_transform(&node.transform)?,
    })
}

/// Converts a [`VoxjTransform`] into a [`TyTransformF64`], validating it:
/// position finite, scale finite and non-zero, rotation finite and non-zero.
/// The rotation is normalized (tolerating a unit quaternion's float error).
fn vox_transform_from_voxj_transform(transform: &VoxjTransform) -> Result<TyTransformF64> {
    let [rotation_x, rotation_y, rotation_z, rotation_w] = transform.rotation;
    let [scale_x, scale_y, scale_z] = transform.scale;

    for value in transform.position {
        if !value.is_finite() {
            return Err(Error::invalid(format!(
                "transform position component {value} must be finite"
            )));
        }
    }

    for value in [scale_x, scale_y, scale_z] {
        if !value.is_finite() || value == 0.0 {
            return Err(Error::invalid(format!(
                "transform scale component {value} must be finite and non-zero"
            )));
        }
    }

    for value in [rotation_x, rotation_y, rotation_z, rotation_w] {
        if !value.is_finite() {
            return Err(Error::invalid(format!(
                "transform rotation component {value} must be finite"
            )));
        }
    }
    let magnitude = (rotation_x * rotation_x
        + rotation_y * rotation_y
        + rotation_z * rotation_z
        + rotation_w * rotation_w)
        .sqrt();
    if magnitude == 0.0 {
        return Err(Error::invalid(
            "transform rotation quaternion must not be zero".to_owned(),
        ));
    }

    Ok(TyTransformF64::new(
        TyVector3F64::from_array(transform.position),
        TyQuaternionF64::from_xyzw(
            rotation_x / magnitude,
            rotation_y / magnitude,
            rotation_z / magnitude,
            rotation_w / magnitude,
        ),
        TyVector3F64::new(scale_x, scale_y, scale_z),
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        EditStateMode, VoxjVoxExt, VoxjVoxMain, VoxjWriteOptions, from_voxj_file,
        from_voxj_file::{
            vox_hierarchy_node_from_voxj_hierarchy_node, vox_object_from_voxj_decoded_object,
            vox_palette_from_voxj_palette,
        },
        to_voxj_file, vox_map_from_voxj_map,
    };
    use branded_id::U32Id;
    use std::{collections::BTreeSet, f64::consts::FRAC_1_SQRT_2};
    use voxj::{
        VoxjEditObject, VoxjEditState, VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjMap,
        VoxjMapEntry, VoxjObject, VoxjPalette, VoxjPositionBlock, VoxjProperty, VoxjRuntimeState,
        VoxjSampleBlock, VoxjTransform, VoxjValue, VoxjValuePool,
        objects::{VoxjDecodedObject, decode_voxj_object, voxj_palette_material_counts},
    };
    use voxj_codec::DependenciesImpl;

    /// A `float` value pool of ascending values `0.0 ..= (n - 1)`, so a
    /// material reading value-index `m` resolves to `m` as a float.
    fn numbered_value_pool(n: usize) -> VoxjValuePool {
        VoxjValuePool::Float((0..n).map(|i| i as f64).collect())
    }

    /// A one-property palette over `value_pool` with `n` materials, material
    /// `m` reading value-index `m`, one row `[m]` per material.
    fn numbered_palette(name: &str, value_pool: usize, n: usize) -> VoxjPalette {
        VoxjPalette {
            properties: vec![property(name, value_pool)],
            materials: (0..n).map(|m| vec![m]).collect(),
        }
    }

    fn property(name: &str, value_pool: usize) -> VoxjProperty {
        VoxjProperty {
            name: name.to_owned(),
            value_pool,
        }
    }

    /// One object holding raw-json position and sample blocks, the readable
    /// form the fixtures author geometry in. `samples` is one row of material
    /// indices per voxel, one entry per layer; transposed into the per-layer
    /// channels the sample block stores, their count read off the row arity.
    fn object(
        name: &str,
        layers: Vec<usize>,
        bounds: [u32; 3],
        positions: Vec<[u32; 3]>,
        samples: Vec<Vec<u32>>,
    ) -> VoxjObject {
        let channel_count = samples.first().map_or(0, Vec::len);
        let channels = (0..channel_count)
            .map(|p| samples.iter().map(|row| row[p]).collect())
            .collect();
        VoxjObject {
            name: name.to_owned(),
            layers,
            bounds,
            origin: [0, 0, 0],
            voxel_positions: VoxjPositionBlock::RawJson(positions),
            voxel_samples: VoxjSampleBlock::RawJson(channels),
        }
    }

    /// A document exercising every field.
    fn sample_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![
                        // value pool 0: six base values, bound by palette 0.
                        numbered_value_pool(6),
                        // value pool 1: metallic values.
                        VoxjValuePool::Float(vec![0.0, 0.5, 1.0]),
                        // value pool 2: ior values.
                        VoxjValuePool::Float(vec![1.5, 2.0]),
                    ],
                    palettes: vec![
                        numbered_palette("baseColor", 0, 6),
                        // Two properties over the scalar value pools, one row
                        // per material: material 0 = { metallic: 0.0,
                        // ior: 1.5 }, material 1 = { metallic: 0.5,
                        // ior: 2.0 }.
                        VoxjPalette {
                            properties: vec![property("metallic", 1), property("ior", 2)],
                            materials: vec![vec![0, 0], vec![1, 1]],
                        },
                    ],
                    objects: vec![
                        object(
                            "sparse",
                            vec![0],
                            [4, 4, 4],
                            vec![[0, 0, 0], [3, 1, 2], [1, 3, 0], [2, 2, 3]],
                            vec![vec![1], vec![0], vec![5], vec![2]],
                        ),
                        // Two layers sharing palette 1: layers do not merge, so
                        // each voxel samples a material index per layer.
                        object(
                            "tight",
                            vec![1, 1],
                            [2, 1, 1],
                            vec![[0, 0, 0], [1, 0, 0]],
                            vec![vec![1, 0], vec![0, 1]],
                        ),
                        object(
                            "no-palette",
                            Vec::new(),
                            [3, 1, 2],
                            vec![[0, 0, 0], [2, 0, 1]],
                            vec![Vec::new(), Vec::new()],
                        ),
                    ],
                    nodes: vec![
                        VoxjHierarchyNode {
                            name: "group".to_owned(),
                            child_nodes: vec![1],
                            child_objects: vec![0],
                            transform: VoxjTransform {
                                position: [1.0, 2.0, 3.0],
                                rotation: [0.0, 0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2],
                                scale: [2.0, 2.0, 2.0],
                            },
                        },
                        VoxjHierarchyNode {
                            name: "leaf".to_owned(),
                            child_nodes: Vec::new(),
                            child_objects: vec![1],
                            transform: VoxjTransform {
                                position: [0.0, 0.0, 0.0],
                                rotation: [0.0, 0.0, 0.0, 1.0],
                                scale: [1.0, 1.0, 1.0],
                            },
                        },
                    ],
                    root_nodes: vec![0],
                },
                edit_state: None,
                ext: Some(VoxjMap::new(vec![VoxjMapEntry {
                    key: "vendor".to_owned(),
                    value: VoxjValue::Array(vec![
                        VoxjValue::Number(1.0),
                        VoxjValue::Bool(true),
                        VoxjValue::Null,
                        VoxjValue::Text("x".to_owned()),
                    ]),
                }])),
            },
        }
    }

    /// The `(position, samples)` pairs of an object, order-independent: the
    /// dense grid re-emits voxels in raster order and the block encodings
    /// reorder them again. The blocks decode against the document palettes.
    fn voxel_set(object: &VoxjObject, palettes: &[VoxjPalette]) -> BTreeSet<([u32; 3], Vec<u32>)> {
        let material_counts = voxj_palette_material_counts(&object.layers, palettes).unwrap();
        let decoded = decode_voxj_object(&DependenciesImpl, object, &material_counts).unwrap();
        decoded
            .positions
            .iter()
            .copied()
            .zip(decoded.samples.iter().cloned())
            .collect()
    }

    fn assert_file_eq(got: &VoxjFile, want: &VoxjFile) {
        assert_eq!(got.main.edit_state, want.main.edit_state);
        assert_eq!(got.main.ext, want.main.ext);
        let (got, want) = (&got.main.runtime_state, &want.main.runtime_state);
        assert_eq!(got.value_pools, want.value_pools);
        assert_eq!(got.palettes, want.palettes);
        assert_eq!(got.nodes, want.nodes);
        assert_eq!(got.root_nodes, want.root_nodes);
        assert_eq!(got.objects.len(), want.objects.len());
        for (got_object, want_object) in got.objects.iter().zip(&want.objects) {
            assert_eq!(got_object.name, want_object.name);
            assert_eq!(got_object.layers, want_object.layers);
            assert_eq!(got_object.bounds, want_object.bounds);
            assert_eq!(
                voxel_set(got_object, &got.palettes),
                voxel_set(want_object, &want.palettes)
            );
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        assert_file_eq(
            &to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap(),
            &file,
        );
    }

    /// A one-object document whose edit grid is larger than its runtime grid,
    /// so the object carries margin.
    fn margin_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(2)],
                    objects: vec![object(
                        "o",
                        vec![0],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![0], vec![1]],
                    )],
                    palettes: vec![numbered_palette("baseColor", 0, 2)],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: Some(VoxjEditState {
                    objects: vec![VoxjEditObject {
                        bounds: [4, 2, 2],
                        origin: [-1, 0, 0],
                    }],
                }),
                ext: None,
            },
        }
    }

    /// A document whose edit grid differs from the runtime grid (has margin)
    /// round-trips the edit state through the VoxMain and back.
    #[test]
    fn round_trips_edit_state() {
        let file = margin_file();
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        assert_file_eq(
            &to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap(),
            &file,
        );
    }

    /// `EditStateMode::Always` records the edit state even when every object is
    /// already tight, one entry per object holding its build volume.
    #[test]
    fn always_records_edit_state_when_tight() {
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &sample_file()).unwrap();
        // Auto omits it: every object in the fixture is already tight.
        assert_eq!(
            to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default())
                .unwrap()
                .main
                .edit_state,
            None
        );

        let options = VoxjWriteOptions {
            edit_state: EditStateMode::Always,
            ..Default::default()
        };
        let file = to_voxj_file(&DependenciesImpl, &main, &options).unwrap();
        assert_eq!(
            file.main.edit_state,
            Some(VoxjEditState {
                objects: vec![
                    VoxjEditObject {
                        bounds: [4, 4, 4],
                        origin: [0, 0, 0],
                    },
                    VoxjEditObject {
                        bounds: [2, 1, 1],
                        origin: [0, 0, 0],
                    },
                    VoxjEditObject {
                        bounds: [3, 1, 2],
                        origin: [0, 0, 0],
                    },
                ],
            })
        );
    }

    /// `EditStateMode::Never` drops the edit state even when an object carries
    /// margin, so its build volume is not recorded.
    #[test]
    fn never_discards_edit_state_with_margin() {
        let file = margin_file();
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        // Auto would emit it because the object carries margin.
        assert!(
            to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default())
                .unwrap()
                .main
                .edit_state
                .is_some()
        );

        let options = VoxjWriteOptions {
            edit_state: EditStateMode::Never,
            ..Default::default()
        };
        let got = to_voxj_file(&DependenciesImpl, &main, &options).unwrap();
        assert_eq!(got.main.edit_state, None);
    }

    /// `ext: false` drops the user-defined ext block. The default keeps it.
    #[test]
    fn ext_false_drops_the_ext_block() {
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &sample_file()).unwrap();
        assert!(
            to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default())
                .unwrap()
                .main
                .ext
                .is_some()
        );
        let options = VoxjWriteOptions {
            ext: false,
            ..Default::default()
        };
        let dropped = to_voxj_file(&DependenciesImpl, &main, &options).unwrap();
        assert_eq!(dropped.main.ext, None);
    }

    /// The typed loader keeps the parsed block. A document with no block loads
    /// an empty ext.
    #[test]
    fn the_typed_loader_keeps_the_block_and_none_loads_empty() {
        let file = sample_file();

        let want = VoxjVoxExt::new(
            vox_map_from_voxj_map(file.main.ext.as_ref().unwrap())
                .unwrap()
                .into_entries(),
        );

        let main = from_voxj_file(&DependenciesImpl, &file).unwrap();

        assert_eq!(main.ext(), &want);

        let main = from_voxj_file(&DependenciesImpl, &margin_file()).unwrap();

        assert_eq!(main.ext(), &VoxjVoxExt::default());
    }

    /// `{}` and no block both load an empty ext and write no block.
    #[test]
    fn an_empty_block_and_none_both_write_no_block() {
        let mut file = margin_file();

        file.main.ext = Some(VoxjMap::default());

        let main = from_voxj_file(&DependenciesImpl, &file).unwrap();

        assert_eq!(main.ext(), &VoxjVoxExt::default());

        assert_eq!(
            to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default())
                .unwrap()
                .main
                .ext,
            None
        );

        let main = from_voxj_file(&DependenciesImpl, &margin_file()).unwrap();

        assert_eq!(
            to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default())
                .unwrap()
                .main
                .ext,
            None
        );
    }

    #[test]
    fn gc_on_a_loaded_state_preserves_the_round_trip() {
        let file = sample_file();
        let mut main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        // A freshly loaded state is already contiguous, so gc leaves the saved
        // document unchanged.
        main.gc().unwrap();
        assert_file_eq(
            &to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap(),
            &file,
        );
    }

    #[test]
    fn rejects_position_outside_bounds() {
        let mut file = sample_file();
        file.main.runtime_state.objects[1].voxel_positions =
            VoxjPositionBlock::RawJson(vec![[9, 0, 0], [1, 0, 0]]);
        assert!(from_voxj_file(&DependenciesImpl, &file).is_err());
    }

    #[test]
    fn rejects_a_root_index_that_would_wrap_onto_a_node() {
        let mut file = sample_file();
        file.main.runtime_state.root_nodes[0] += 1 << 32;

        let error = from_voxj_file(&DependenciesImpl, &file).unwrap_err();

        assert!(error.to_string().contains("past the id space"), "{error}");
    }

    #[test]
    fn rejects_a_child_object_index_that_would_wrap_onto_an_object() {
        let mut file = sample_file();
        let node = file
            .main
            .runtime_state
            .nodes
            .iter_mut()
            .find(|node| !node.child_objects.is_empty())
            .unwrap();
        node.child_objects[0] += 1 << 32;

        let error = from_voxj_file(&DependenciesImpl, &file).unwrap_err();

        assert!(error.to_string().contains("past the id space"), "{error}");
    }

    #[test]
    fn rejects_a_value_pool_reference_that_would_wrap_onto_a_pool() {
        let mut file = sample_file();
        file.main.runtime_state.palettes[0].properties[0].value_pool += 1 << 32;

        let error = from_voxj_file(&DependenciesImpl, &file).unwrap_err();

        assert!(error.to_string().contains("past the id space"), "{error}");
    }

    #[test]
    fn rejects_out_of_range_sample() {
        let mut file = sample_file();
        // One channel (object 0 has one layer); the first voxel samples
        // material 99, out of range for palette 0's six materials.
        file.main.runtime_state.objects[0].voxel_samples =
            VoxjSampleBlock::RawJson(vec![vec![99, 0, 5, 2]]);
        assert!(from_voxj_file(&DependenciesImpl, &file).is_err());
    }

    /// A voxelless object (tight bounds `[0, 0, 0]`) still carries one empty
    /// channel per layer and round-trips rather than being rejected.
    #[test]
    fn round_trips_a_voxelless_object_with_a_layer() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(1)],
                    objects: vec![VoxjObject {
                        name: "empty-ref".to_owned(),
                        layers: vec![0],
                        bounds: [0, 0, 0],
                        origin: [0, 0, 0],
                        voxel_positions: VoxjPositionBlock::RawJson(Vec::new()),
                        voxel_samples: VoxjSampleBlock::RawJson(vec![Vec::new()]),
                    }],
                    palettes: vec![numbered_palette("baseColor", 0, 1)],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        assert_file_eq(
            &to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap(),
            &file,
        );
    }

    /// Sibling variant palettes round-trip: both share the same value pools and
    /// differ in one column, and the object layered over them carries one
    /// channel per layer.
    #[test]
    fn round_trips_sibling_variant_palettes() {
        let variant = |strength: usize| VoxjPalette {
            properties: vec![property("baseColor", 0), property("emissiveStrength", 1)],
            materials: vec![vec![0, strength], vec![1, strength]],
        };
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![numbered_value_pool(2), numbered_value_pool(3)],
                    objects: vec![object(
                        "lamp",
                        vec![0, 1],
                        [2, 1, 1],
                        vec![[0, 0, 0], [1, 0, 0]],
                        vec![vec![0, 0], vec![1, 1]],
                    )],
                    palettes: vec![variant(0), variant(2)],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        assert_file_eq(
            &to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap(),
            &file,
        );
    }

    /// Every vector kind maps one to one through the vox state: the values come
    /// back bit-identical, colors and plain vectors alike.
    #[test]
    fn round_trips_vector_value_pools() {
        let file = vector_kind_file();
        let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
        let written = to_voxj_file(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap();
        assert_eq!(
            written.main.runtime_state.value_pools,
            file.main.runtime_state.value_pools
        );
    }

    /// A one-voxel object over a palette binding one property per vector kind.
    fn vector_kind_file() -> VoxjFile {
        VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: vec![
                        VoxjValuePool::Vec2Float(vec![[0.25, 0.75]]),
                        VoxjValuePool::Vec2Int(vec![[3, 7]]),
                        VoxjValuePool::Vec3Float(vec![[2.5, 0.0, 1.0]]),
                        VoxjValuePool::Vec3Int(vec![[-1, 0, 1]]),
                        VoxjValuePool::Vec4Float(vec![
                            [1.0, 0.0, 0.0, 1.0],
                            [0.0, 1.0, 0.0, 128.0 / 255.0],
                        ]),
                        VoxjValuePool::Vec4Int(vec![[0, 1, 2, 3]]),
                    ],
                    objects: vec![object(
                        "o",
                        vec![0],
                        [1, 1, 1],
                        vec![[0, 0, 0]],
                        vec![vec![0]],
                    )],
                    palettes: vec![VoxjPalette {
                        properties: vec![
                            property("customUv", 0),
                            property("customCell", 1),
                            property("emissiveColor", 2),
                            property("customAxis", 3),
                            property("baseColor", 4),
                            property("customTint", 5),
                        ],
                        materials: vec![vec![0, 0, 0, 0, 0, 0], vec![0, 0, 0, 0, 1, 0]],
                    }],
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        }
    }

    /// A sparse object with huge bounds is rejected at the volume check, before
    /// it can force a dense multi-gigabyte allocation.
    #[test]
    fn rejects_oversized_dense_grid() {
        let file = VoxjFile {
            version: 1,
            main: VoxjMain {
                runtime_state: VoxjRuntimeState {
                    value_pools: Vec::new(),
                    objects: vec![object(
                        "huge",
                        Vec::new(),
                        [1024, 1024, 1024],
                        Vec::new(),
                        Vec::new(),
                    )],
                    palettes: Vec::new(),
                    nodes: Vec::new(),
                    root_nodes: Vec::new(),
                },
                edit_state: None,
                ext: None,
            },
        };
        assert!(from_voxj_file(&DependenciesImpl, &file).is_err());
    }

    fn node_with_transform(transform: VoxjTransform) -> VoxjHierarchyNode {
        VoxjHierarchyNode {
            name: "n".to_owned(),
            child_nodes: Vec::new(),
            child_objects: Vec::new(),
            transform,
        }
    }

    #[test]
    fn rejects_zero_scale_and_non_finite_components() {
        let zero_scale = node_with_transform(VoxjTransform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 0.0, 1.0],
        });
        assert!(vox_hierarchy_node_from_voxj_hierarchy_node(&zero_scale).is_err());

        let nan_position = node_with_transform(VoxjTransform {
            position: [f64::NAN, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        });
        assert!(vox_hierarchy_node_from_voxj_hierarchy_node(&nan_position).is_err());
    }

    #[test]
    fn normalizes_a_non_unit_rotation() {
        let node = node_with_transform(VoxjTransform {
            position: [0.0, 0.0, 0.0],
            // Magnitude 2; should normalize to the unit identity (w = 1).
            rotation: [0.0, 0.0, 0.0, 2.0],
            scale: [1.0, 1.0, 1.0],
        });
        let rotation = vox_hierarchy_node_from_voxj_hierarchy_node(&node)
            .unwrap()
            .transform
            .rotation;
        assert_eq!(
            (rotation.x, rotation.y, rotation.z, rotation.w),
            (0.0, 0.0, 0.0, 1.0)
        );
    }

    #[test]
    fn maps_material_rows_one_to_one() {
        let palette = VoxjPalette {
            properties: vec![property("baseColor", 0), property("metallic", 1)],
            materials: vec![vec![0, 2], vec![1, 0], vec![2, 1]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.property_count(), 2);
        assert_eq!(out.material_count(), 3);

        let base_property_id = out.property_id_by_name("baseColor").unwrap();
        let metallic_property_id = out.property_id_by_name("metallic").unwrap();
        let material_2_id = out.iter_materials().nth(2).unwrap();
        // Material 2 reads value id 2 for base color and 1 for metallic.
        assert_eq!(
            out.value_id(material_2_id, base_property_id),
            Some(U32Id::from_u32(2))
        );
        assert_eq!(
            out.value_id(material_2_id, metallic_property_id),
            Some(U32Id::from_u32(1))
        );
    }

    #[test]
    fn reads_a_property_less_palette_keeping_its_material_count() {
        // With no properties every row is empty; each mints a material with no
        // value ids.
        let palette = VoxjPalette {
            properties: vec![],
            materials: vec![vec![], vec![], vec![]],
        };
        let out = vox_palette_from_voxj_palette(&palette).unwrap();
        assert_eq!(out.property_count(), 0);
        assert_eq!(out.material_count(), 3);
    }

    #[test]
    fn rejects_a_non_empty_row_without_properties() {
        let palette = VoxjPalette {
            properties: vec![],
            materials: vec![vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_duplicate_property_name() {
        let palette = VoxjPalette {
            properties: vec![property("rgba", 0), property("rgba", 1)],
            materials: vec![vec![0, 0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_short_material_row() {
        let palette = VoxjPalette {
            properties: vec![property("a", 0), property("b", 1)],
            materials: vec![vec![0, 1], vec![0]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_value_index_past_the_id_space() {
        // A `usize` index past `u32` would wrap onto a real value and bind the
        // material to a value the file never named.
        let palette = VoxjPalette {
            properties: vec![property("a", 0)],
            materials: vec![vec![1usize << 32]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[test]
    fn rejects_a_layer_index_past_the_id_space() {
        // A `usize` index past `u32` would wrap onto a real palette.
        let object = VoxjDecodedObject {
            name: "o".to_owned(),
            layers: vec![1usize << 32],
            bounds: [1, 1, 1],
            ..Default::default()
        };

        assert!(vox_object_from_voxj_decoded_object(&object, None).is_err());
    }

    #[test]
    fn rejects_a_long_material_row() {
        let palette = VoxjPalette {
            properties: vec![property("a", 0), property("b", 1)],
            materials: vec![vec![0, 1], vec![0, 1, 2]],
        };
        assert!(vox_palette_from_voxj_palette(&palette).is_err());
    }

    #[cfg(feature = "codec")]
    mod codec {
        use crate::{
            VoxjVoxMain, VoxjWriteOptions,
            codec::{from_voxj_bytes, to_voxj_bytes, to_voxjz_bytes},
            from_voxj_file,
            from_voxj_file::tests::{
                assert_file_eq, numbered_palette, numbered_value_pool, object, sample_file,
            },
            to_voxj_file,
        };
        use branded_id::U32Id;
        use voxcore::{BVoxHierarchyNode, BVoxLayer, BVoxObject, BVoxPalette, VoxHierarchyNode};
        use voxj::{VoxjFile, VoxjHierarchyNode, VoxjMain, VoxjRuntimeState};
        use voxj_codec::DependenciesImpl;

        /// [`sample_file`] after removing the "tight" object (id 1) and the
        /// palette only it referenced (id 1), then compacting: the two
        /// survivors renumber to objects 0 and 1, the lone palette stays 0, and
        /// the "leaf" node loses its reference to the removed object. The value
        /// pools are untouched: there is no value-pool removal, so the
        /// now-unreferenced value pools stay in place.
        fn sample_file_without_tight() -> VoxjFile {
            let base = sample_file();
            VoxjFile {
                version: base.version,
                main: VoxjMain {
                    runtime_state: VoxjRuntimeState {
                        value_pools: base.main.runtime_state.value_pools.clone(),
                        objects: vec![
                            base.main.runtime_state.objects[0].clone(),
                            base.main.runtime_state.objects[2].clone(),
                        ],
                        palettes: vec![base.main.runtime_state.palettes[0].clone()],
                        nodes: vec![
                            base.main.runtime_state.nodes[0].clone(),
                            VoxjHierarchyNode {
                                child_objects: Vec::new(),
                                ..base.main.runtime_state.nodes[1].clone()
                            },
                        ],
                        root_nodes: vec![0],
                    },
                    edit_state: None,
                    ext: base.main.ext.clone(),
                },
            }
        }

        #[test]
        fn round_trips_through_voxj_bytes() {
            let file = sample_file();
            let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
            let bytes =
                to_voxj_bytes(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap();
            let reloaded: VoxjVoxMain = from_voxj_bytes(&DependenciesImpl, &bytes).unwrap();
            assert_file_eq(
                &to_voxj_file(&DependenciesImpl, &reloaded, &VoxjWriteOptions::default()).unwrap(),
                &file,
            );
        }

        #[test]
        fn round_trips_through_voxjz_bytes() {
            let file = sample_file();
            let main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();
            let bytes =
                to_voxjz_bytes(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap();
            let reloaded: VoxjVoxMain = from_voxj_bytes(&DependenciesImpl, &bytes).unwrap();
            assert_file_eq(
                &to_voxj_file(&DependenciesImpl, &reloaded, &VoxjWriteOptions::default()).unwrap(),
                &file,
            );
        }

        #[test]
        fn remove_then_gc_round_trips_through_bytes() {
            let mut main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &sample_file()).unwrap();

            // The "leaf" node places the "tight" object. Nodes are frozen, so
            // removing the object means rebuilding the hierarchy: clear the
            // roots, release the nodes top-down, and re-retain them once the
            // object is gone.
            let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);
            let leaf_id = U32Id::<BVoxHierarchyNode>::from_u32(1);
            let group = main.hierarchy_node(group_id).unwrap().clone();
            let leaf = main.hierarchy_node(leaf_id).unwrap().clone();
            main.set_root_hierarchy_node_ids(Vec::new()).unwrap();
            assert_eq!(main.release_hierarchy_node(group_id), Ok(()));
            assert_eq!(main.release_hierarchy_node(leaf_id), Ok(()));

            // Remove the "tight" object and the palette only it referenced,
            // then compact so the save numbers entities by listing index again.
            assert_eq!(
                main.release_object(U32Id::<BVoxObject>::from_u32(1)),
                Ok(())
            );
            assert_eq!(
                main.release_palette(U32Id::<BVoxPalette>::from_u32(1)),
                Ok(())
            );
            main.gc().unwrap();

            // Rebuild the nodes on the compacted pool: "group" keeps its child
            // list, which references the rebuilt "leaf" by the id the batch
            // assigns it, and "leaf" no longer places the removed object.
            let node_ids = main
                .retain_hierarchy_nodes(vec![
                    group,
                    VoxHierarchyNode {
                        child_object_ids: Vec::new(),
                        ..leaf
                    },
                ])
                .unwrap();
            main.set_root_hierarchy_node_ids(vec![node_ids[0]]).unwrap();

            let bytes =
                to_voxj_bytes(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap();
            let reloaded: VoxjVoxMain = from_voxj_bytes(&DependenciesImpl, &bytes).unwrap();
            assert_file_eq(
                &to_voxj_file(&DependenciesImpl, &reloaded, &VoxjWriteOptions::default()).unwrap(),
                &sample_file_without_tight(),
            );
        }

        /// Removing the first layer keeps the surviving layers' relative order
        /// through a gc and a save: the document still lists the second and
        /// third layers' palettes and samples in their original order. Removing
        /// the first of three is the smallest case a swap-remove would get
        /// wrong, listing the third layer before the second.
        #[test]
        fn remove_first_layer_then_gc_keeps_layer_order() {
            let palettes = vec![
                numbered_palette("baseColor", 0, 2),
                numbered_palette("baseColor", 0, 2),
                numbered_palette("baseColor", 0, 2),
            ];
            let file = VoxjFile {
                version: 1,
                main: VoxjMain {
                    runtime_state: VoxjRuntimeState {
                        value_pools: vec![numbered_value_pool(2)],
                        objects: vec![object(
                            "o",
                            vec![0, 1, 2],
                            [2, 1, 1],
                            vec![[0, 0, 0], [1, 0, 0]],
                            vec![vec![0, 1, 0], vec![1, 0, 1]],
                        )],
                        palettes: palettes.clone(),
                        nodes: Vec::new(),
                        root_nodes: Vec::new(),
                    },
                    edit_state: None,
                    ext: None,
                },
            };
            let mut main: VoxjVoxMain = from_voxj_file(&DependenciesImpl, &file).unwrap();

            // Layer ids follow the listing on load, so the first layer is id 0.
            let object_id = U32Id::<BVoxObject>::from_u32(0);
            assert_eq!(
                main.release_layer(object_id, U32Id::<BVoxLayer>::from_u32(0)),
                Ok(())
            );
            main.gc().unwrap();
            main.validate().unwrap();

            let bytes =
                to_voxj_bytes(&DependenciesImpl, &main, &VoxjWriteOptions::default()).unwrap();
            let reloaded: VoxjVoxMain = from_voxj_bytes(&DependenciesImpl, &bytes).unwrap();
            let expected = VoxjFile {
                version: 1,
                main: VoxjMain {
                    runtime_state: VoxjRuntimeState {
                        value_pools: vec![numbered_value_pool(2)],
                        objects: vec![object(
                            "o",
                            vec![1, 2],
                            [2, 1, 1],
                            vec![[0, 0, 0], [1, 0, 0]],
                            vec![vec![1, 0], vec![0, 1]],
                        )],
                        palettes,
                        nodes: Vec::new(),
                        root_nodes: Vec::new(),
                    },
                    edit_state: None,
                    ext: None,
                },
            };
            assert_file_eq(
                &to_voxj_file(&DependenciesImpl, &reloaded, &VoxjWriteOptions::default()).unwrap(),
                &expected,
            );
        }
    }
}
