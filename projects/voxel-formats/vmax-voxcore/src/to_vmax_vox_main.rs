use crate::{
    FALLBACK_CONTENT_VERSION, MATERIAL_SLOTS, Result, SYNTH_CAMERA, VMaxExt, VMaxExtPalette,
    VMaxVoxMain, synthesized_node, synthesized_object_state,
};
use branded_id::{IdRange, U32Id};
use std::collections::{HashMap, HashSet};
use vmax::VMaxSceneJsonFile;
use voxcore::{
    BVoxHierarchyNode, BVoxLayer, BVoxMaterial, BVoxObject, BVoxPalette, BVoxProperty,
    BVoxValuePool, BVoxValuePoolValue, VoxHierarchyNode, VoxMain, VoxPalette, VoxValueColumn,
    VoxValuePool, VoxValuePoolValues,
    color::ColorValues,
    material::{BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH},
};

/// Gives a bare state a synthesized [`VMaxExt`], the state
/// [`to_vmax_file`](crate::to_vmax_file()) writes as a document synthesized
/// from the scene. Voxel Max models a tree, so the hierarchy becomes one first.
/// A node reached along several paths is cloned per extra path, the way voxcore
/// composes a node's placement along every path to it, and a node reached from
/// no root is released because voxcore never places it. Voxel Max parents
/// objects only under groups, so a node placing both objects and child nodes
/// moves its objects onto a child node of their own. A Voxel Max palette
/// holds every property but `baseColor` and `emissiveColor` one value per
/// material slot, so a palette holding them otherwise, such as one that pools
/// each property's distinct values apart, is laid out in slots next. A
/// palette several objects share that needs more slots than a Voxel Max
/// palette holds splits first into one palette per set of materials an object
/// samples. Every material keeps the values it draws. Each node, palette, and
/// object then takes the entry the retain hooks would build for it: fresh ids,
/// the node's rotation, the default anchor tokens, no exact material list, and
/// a camera framed on the object. The scene takes the fallback version and the
/// neutral camera.
///
/// Lossy only on the material palette name, which stays empty.
pub fn to_vmax_vox_main(mut main: VoxMain<()>) -> Result<VMaxVoxMain> {
    unshare(&mut main)?;
    group_objects_beside_nodes(&mut main)?;
    split_crowded_palettes(&mut main)?;
    lay_out_material_slots(&mut main)?;

    let mut ext = VMaxExt {
        scene: VMaxSceneJsonFile {
            v: FALLBACK_CONTENT_VERSION,
            cam: Some(SYNTH_CAMERA),
            ..Default::default()
        },
        ..Default::default()
    };

    for (node_id, node) in main.iter_hierarchy_nodes() {
        let entry = synthesized_node(&ext, node);
        ext.hierarchy_nodes.insert(node_id, entry);
    }

    for (palette_id, _) in main.iter_palettes() {
        ext.palettes.insert(palette_id, VMaxExtPalette::default());
    }

    for (object_id, object) in main.iter_objects() {
        let entry = synthesized_object_state(&ext, object);
        ext.object_states.insert(object_id, entry);
    }

    Ok(main.put_ext(ext))
}

/// Reshapes the hierarchy into a tree the roots reach in full. Walks from
/// each root in order. A node's first visit keeps it. A later visit clones
/// the node and its subtree, so a node shared by several parents, or one that
/// is both a root and a child, is placed once per path. A node no walk
/// reaches is released.
fn unshare(main: &mut VoxMain<()>) -> Result<()> {
    let mut reached = HashSet::new();
    let mut root_ids = Vec::new();

    for root_id in main.root_hierarchy_node_ids().to_vec() {
        root_ids.push(visit(main, root_id, &mut reached)?);
    }

    main.set_root_hierarchy_node_ids(root_ids)?;

    let unreached: Vec<_> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .filter(|node_id| !reached.contains(node_id))
        .collect();

    // An unreached node's parents are all unreached. Unlinking them first
    // lets each release in any order.
    for &node_id in &unreached {
        let child_object_ids = main
            .hierarchy_node(node_id)
            .expect("a listed node")
            .child_object_ids
            .clone();
        main.set_hierarchy_node_children(node_id, Vec::new(), child_object_ids)?;
    }

    for node_id in unreached {
        main.release_hierarchy_node(node_id)?;
    }

    Ok(())
}

/// Visits `node_id` along one more path and returns the node that path
/// places: the node itself on its first visit, a clone after. The children
/// are visited the same way, so a clone's subtree is cloned with it.
fn visit(
    main: &mut VoxMain<()>,
    node_id: U32Id<BVoxHierarchyNode>,
    reached: &mut HashSet<U32Id<BVoxHierarchyNode>>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    let first = reached.insert(node_id);
    let mut node = main
        .hierarchy_node(node_id)
        .expect("a root or child is a listed node")
        .clone();

    let mut child_node_ids = Vec::with_capacity(node.child_node_ids.len());
    for &child_id in &node.child_node_ids {
        child_node_ids.push(visit(main, child_id, reached)?);
    }
    node.child_node_ids = child_node_ids;

    if first {
        main.set_hierarchy_node_children(node_id, node.child_node_ids, node.child_object_ids)?;
        return Ok(node_id);
    }

    let clone_id = main.retain_hierarchy_node(node)?;
    reached.insert(clone_id);
    Ok(clone_id)
}

/// Gives each node placing both objects and child nodes a first child node
/// placing its objects. The writer writes a node placing objects as those
/// objects, and Voxel Max parents an object only under a group, so the node's
/// child nodes would otherwise name an object as their parent. The new node
/// takes the node's name and the identity transform, so every object keeps its
/// placement.
fn group_objects_beside_nodes(main: &mut VoxMain<()>) -> Result<()> {
    let mixed_ids: Vec<_> = main
        .iter_hierarchy_nodes()
        .filter(|(_, node)| !node.child_object_ids.is_empty() && !node.child_node_ids.is_empty())
        .map(|(node_id, _)| node_id)
        .collect();

    for node_id in mixed_ids {
        let node = main.hierarchy_node(node_id).expect("a listed node").clone();
        let objects_id = main.retain_hierarchy_node(VoxHierarchyNode {
            name: node.name,
            child_object_ids: node.child_object_ids,
            ..Default::default()
        })?;

        let mut child_node_ids = vec![objects_id];
        child_node_ids.extend(node.child_node_ids);
        main.set_hierarchy_node_children(node_id, child_node_ids, Vec::new())?;
    }

    Ok(())
}

/// Splits each palette that several objects layer and that needs more than
/// [`MATERIAL_SLOTS`] slots, as [`lay_out_material_slots`] counts them. A Voxel
/// Max object reads a palette of its own, so each set of materials an object
/// samples takes a palette, which objects sampling that same set share. The
/// palette keeps every property and its value pool and holds those materials
/// in their order. A palette holding a material no live voxel samples stays
/// whole, since a split would drop that material, and errors at the write.
fn split_crowded_palettes(main: &mut VoxMain<()>) -> Result<()> {
    let palette_ids: Vec<_> = main
        .iter_palettes()
        .map(|(palette_id, _)| palette_id)
        .collect();

    for palette_id in palette_ids {
        split_crowded_palette(main, palette_id)?;
    }

    Ok(())
}

/// Splits palette `palette_id` as [`split_crowded_palettes`] describes.
fn split_crowded_palette(main: &mut VoxMain<()>, palette_id: U32Id<BVoxPalette>) -> Result<()> {
    let palette = main.palette(palette_id).expect("a listed palette");
    if slot_plan(main, palette).slot_material_ids.len() <= MATERIAL_SLOTS {
        return Ok(());
    }

    // Each layer on the palette with the materials it samples, in palette
    // order. Layers sampling the same materials share a split.
    let mut splits: Vec<(Vec<U32Id<BVoxMaterial>>, Vec<_>)> = Vec::new();
    let mut sampled_ids = HashSet::new();
    for (object_id, object) in main.iter_objects() {
        for (layer_id, layer_palette_id) in object.iter_layers() {
            if layer_palette_id != palette_id {
                continue;
            }

            let samples: HashSet<_> = object
                .iter_live_samples(layer_id)
                .expect("an iterated layer is one of the object's layers")
                .map(|(_, material_id)| material_id)
                .collect();
            let material_ids: Vec<_> = palette
                .iter_materials()
                .filter(|material_id| samples.contains(material_id))
                .collect();
            sampled_ids.extend(samples);
            match splits.iter_mut().find(|(ids, _)| *ids == material_ids) {
                Some((_, layers)) => layers.push((object_id, layer_id)),
                None => splits.push((material_ids, vec![(object_id, layer_id)])),
            }
        }
    }
    let layer_count: usize = splits.iter().map(|(_, layers)| layers.len()).sum();
    let whole = palette
        .iter_materials()
        .all(|material_id| sampled_ids.contains(&material_id));
    if layer_count < 2 || !whole {
        return Ok(());
    }

    let properties: Vec<_> = palette
        .iter_properties()
        .map(|(property_id, property)| (property_id, property.name.clone(), property.value_pool_id))
        .collect();
    let mut built = Vec::with_capacity(splits.len());
    for (material_ids, layers) in splits {
        let mut split = VoxPalette::default();
        for (_, name, value_pool_id) in &properties {
            split.retain_property(name.clone(), *value_pool_id)?;
        }

        let mut replacement_ids = HashMap::new();
        for &material_id in &material_ids {
            let value_ids = properties
                .iter()
                .map(|&(property_id, _, _)| {
                    palette
                        .value_id(material_id, property_id)
                        .expect("a live material has a value id for every property")
                })
                .collect();
            replacement_ids.insert(material_id, split.retain_material(value_ids)?);
        }
        built.push((split, replacement_ids, layers));
    }

    for (split, replacement_ids, layers) in built {
        let split_id = main.retain_palette(split)?;
        for (object_id, layer_id) in layers {
            move_layer_to_palette(main, object_id, layer_id, split_id, &replacement_ids)?;
        }
    }

    main.release_palette(palette_id)?;
    Ok(())
}

/// Moves layer `layer_id` of object `object_id` onto palette `palette_id`: a
/// layer on the palette takes its place in the layer order, and each live
/// voxel samples the replacement for the material it sampled.
fn move_layer_to_palette(
    main: &mut VoxMain<()>,
    object_id: U32Id<BVoxObject>,
    layer_id: U32Id<BVoxLayer>,
    palette_id: U32Id<BVoxPalette>,
    replacement_ids: &HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,
) -> Result<()> {
    let object = main.object(object_id).expect("a listed object");
    let layer_ids: Vec<_> = object.iter_layers().map(|(id, _)| id).collect();
    let index = layer_ids
        .iter()
        .position(|&id| id == layer_id)
        .expect("one of the object's layers");
    let samples: Vec<(_, Vec<_>)> = object
        .iter_live()
        .map(|voxel_id| {
            let sample_ids = layer_ids
                .iter()
                .map(|&id| {
                    object
                        .voxel_material(voxel_id, id)
                        .expect("a live voxel samples every layer")
                })
                .collect();
            (voxel_id, sample_ids)
        })
        .collect();
    // An object with no live voxel takes the layer unfilled, since its split
    // holds no material to fill with.
    let moved_id = match samples.first() {
        Some((_, sample_ids)) => {
            main.retain_layer_filled(object_id, palette_id, replacement_ids[&sample_ids[index]])?
        }

        None => main.retain_layer(object_id, palette_id)?,
    };
    for (voxel_id, mut sample_ids) in samples {
        sample_ids.push(replacement_ids[&sample_ids[index]]);
        main.retain_voxel(object_id, voxel_id, &sample_ids)?;
    }
    main.release_layer(object_id, layer_id)?;
    main.move_layer(object_id, moved_id, index)?;

    Ok(())
}

/// Lays each palette's material axis out in Voxel Max's slots, the layout
/// [`to_vmax_file`](crate::to_vmax_file()) reads: every property but
/// `baseColor` and `emissiveColor` holds one value per slot, numbered from
/// zero, and each material draws one slot from all of them. A palette already
/// in slots stays as it is. Any other palette takes one slot per distinct set
/// of material values, numbered in the order its materials first draw them,
/// and its properties bind anew in their order, so every material keeps the
/// values it draws. The color axis keeps its value pools. A value pool no
/// property binds afterwards is released. A palette needing more than
/// [`MATERIAL_SLOTS`] slots still errors at the write.
///
/// Voxel Max glows a slot in each voxel's base color at the slot's
/// `emissiveStrength`, while a voxcore material glows in its `emissiveColor`
/// scaled by that strength. A material whose `emissiveColor` is black over a
/// `baseColor` that is not glows nowhere at any strength, so its slot reads a
/// strength of 0, which looks the same. A palette in slots that places such a
/// material on a glowing slot is laid out anew.
fn lay_out_material_slots(main: &mut VoxMain<()>) -> Result<()> {
    let palette_ids: Vec<_> = main
        .iter_palettes()
        .map(|(palette_id, _)| palette_id)
        .collect();

    let mut unbound_ids = Vec::new();
    for palette_id in palette_ids {
        unbound_ids.extend(lay_out_palette_slots(main, palette_id)?);
    }

    for value_pool_id in unbound_ids {
        let bound = main.iter_palettes().any(|(_, palette)| {
            palette
                .iter_properties()
                .any(|(_, property)| property.value_pool_id == value_pool_id)
        });
        if !bound && main.value_pool(value_pool_id).is_some() {
            main.release_value_pool(value_pool_id)?;
        }
    }

    Ok(())
}

/// How a palette's materials fall into Voxel Max slots under
/// [`lay_out_material_slots`].
struct SlotPlan {
    /// Whether the palette already holds its material axis in these slots.
    in_slots: bool,

    /// Each slot's first material, in slot order.
    slot_material_ids: Vec<U32Id<BVoxMaterial>>,

    /// The slot each material draws, in material order.
    slots: Vec<U32Id<BVoxValuePoolValue>>,

    /// The `emissiveStrength` property when it binds scalars, with the
    /// strength each slot glows at in Voxel Max.
    strengths: Option<(U32Id<BVoxProperty>, Vec<f64>)>,
}

/// The `(property, value pool)` pairs of `palette`'s material axis: every
/// property but `baseColor` and `emissiveColor`, in property order.
fn material_axis(palette: &VoxPalette) -> Vec<(U32Id<BVoxProperty>, U32Id<BVoxValuePool>)> {
    palette
        .iter_properties()
        .filter(|(_, property)| !matches!(property.name.as_str(), BASE_COLOR | EMISSIVE_COLOR))
        .map(|(property_id, property)| (property_id, property.value_pool_id))
        .collect()
}

/// Plans `palette`'s slots as [`lay_out_material_slots`] describes.
fn slot_plan(main: &VoxMain<()>, palette: &VoxPalette) -> SlotPlan {
    let material_axis = material_axis(palette);
    let value_id = |material_id: U32Id<BVoxMaterial>, property_id: U32Id<BVoxProperty>| {
        palette
            .value_id(material_id, property_id)
            .expect("a live material has a value id for every property")
    };

    let unlit_ids = unlit_material_ids(main, palette);
    let strength = palette
        .property_id_by_name(EMISSIVE_STRENGTH)
        .and_then(|property_id| {
            let property = palette.property(property_id).expect("a named property");
            let strengths = main
                .value_pool(property.value_pool_id)
                .expect("a property draws from a live value pool")
                .float_values()?;
            Some((property_id, strengths))
        });
    let stored_strength = |material_id| -> Option<f64> {
        let (property_id, strengths) = strength?;
        let strength = strengths
            .get(value_id(material_id, property_id))
            .expect("a live material draws one of its property's values");
        Some(*strength)
    };
    // The strength `material_id`'s slot glows at in Voxel Max.
    let slot_strength = |material_id| -> Option<f64> {
        let strength = stored_strength(material_id)?;
        Some(match unlit_ids.contains(&material_id) {
            true => 0.0,
            false => strength,
        })
    };

    let lit_on_slot = unlit_ids
        .iter()
        .any(|&material_id| stored_strength(material_id).is_some_and(|strength| strength != 0.0));
    let strength_id = strength.map(|(property_id, _)| property_id);
    let same_slot = |a: U32Id<BVoxMaterial>, b: U32Id<BVoxMaterial>| {
        material_axis.iter().all(|&(property_id, value_pool_id)| {
            if Some(property_id) == strength_id {
                return slot_strength(a) == slot_strength(b);
            }

            same_value(
                main.value_pool(value_pool_id)
                    .expect("a property draws from a live value pool"),
                value_id(a, property_id),
                value_id(b, property_id),
            )
        })
    };

    let mut slot_material_ids: Vec<U32Id<BVoxMaterial>> = Vec::new();
    let mut slots = Vec::new();
    for material_id in palette.iter_materials() {
        let slot = slot_material_ids
            .iter()
            .position(|&slot_material_id| same_slot(slot_material_id, material_id));
        let slot = slot.unwrap_or_else(|| {
            slot_material_ids.push(material_id);
            slot_material_ids.len() - 1
        });
        slots.push(U32Id::from_u32(
            u32::try_from(slot).expect("a slot per material fits a material id"),
        ));
    }

    let strengths = strength_id.map(|property_id| {
        let strengths = slot_material_ids
            .iter()
            .map(|&material_id| slot_strength(material_id).expect("a strength reads a scalar"))
            .collect();
        (property_id, strengths)
    });

    SlotPlan {
        in_slots: in_slots(main, palette, &material_axis) && !lit_on_slot,
        slot_material_ids,
        slots,
        strengths,
    }
}

/// One property of a palette being laid out in slots: the value pool it binds
/// next and the value id each material draws from it, in material order.
struct SlotBinding {
    property_id: U32Id<BVoxProperty>,

    name: String,

    value_pool_id: U32Id<BVoxValuePool>,

    value_ids: Vec<U32Id<BVoxValuePoolValue>>,
}

/// Lays palette `palette_id` out in slots as [`lay_out_material_slots`]
/// describes and returns the value pools its material axis bound before.
fn lay_out_palette_slots(
    main: &mut VoxMain<()>,
    palette_id: U32Id<BVoxPalette>,
) -> Result<Vec<U32Id<BVoxValuePool>>> {
    let palette = main.palette(palette_id).expect("a listed palette");
    let plan = slot_plan(main, palette);
    if plan.in_slots {
        return Ok(Vec::new());
    }

    let material_ids: Vec<_> = palette.iter_materials().collect();
    let material_axis = material_axis(palette);
    let value_id = |material_id: U32Id<BVoxMaterial>, property_id: U32Id<BVoxProperty>| {
        palette
            .value_id(material_id, property_id)
            .expect("a live material has a value id for every property")
    };

    let mut bindings = Vec::new();
    let mut slot_value_pools = Vec::new();
    for (property_id, property) in palette.iter_properties() {
        let name = property.name.clone();
        if !material_axis.iter().any(|&(id, _)| id == property_id) {
            let value_ids = material_ids
                .iter()
                .map(|&material_id| value_id(material_id, property_id))
                .collect();
            bindings.push(SlotBinding {
                property_id,
                name,
                value_pool_id: property.value_pool_id,
                value_ids,
            });
            continue;
        }

        let slot_value_pool = match &plan.strengths {
            Some((strength_id, strengths)) if *strength_id == property_id => {
                VoxValuePool::float(strengths.clone())?
            }

            _ => {
                let slot_value_ids: Vec<_> = plan
                    .slot_material_ids
                    .iter()
                    .map(|&material_id| value_id(material_id, property_id))
                    .collect();
                gathered_value_pool(
                    main.value_pool(property.value_pool_id)
                        .expect("a property draws from a live value pool"),
                    &slot_value_ids,
                )?
            }
        };
        slot_value_pools.push((bindings.len(), slot_value_pool));
        bindings.push(SlotBinding {
            property_id,
            name,
            value_pool_id: property.value_pool_id,
            value_ids: plan.slots.clone(),
        });
    }

    let unbound_ids = material_axis
        .iter()
        .map(|&(_, value_pool_id)| value_pool_id)
        .collect();
    for (index, value_pool) in slot_value_pools {
        bindings[index].value_pool_id = main.retain_value_pool(value_pool);
    }

    for binding in bindings {
        main.release_property(palette_id, binding.property_id)?;
        let property_id = match binding.value_ids.first() {
            Some(&default_value_id) => main.retain_property_filled(
                palette_id,
                binding.name,
                binding.value_pool_id,
                default_value_id,
            )?,

            None => main.retain_property(palette_id, binding.name, binding.value_pool_id)?,
        };
        for (&material_id, &value_id) in material_ids.iter().zip(&binding.value_ids) {
            main.set_material_value(palette_id, material_id, property_id, value_id)?;
        }
    }

    Ok(unbound_ids)
}

/// The materials of `palette` that glow nowhere but would glow on a glowing
/// Voxel Max slot: each draws a black `emissiveColor` over a `baseColor` that
/// is not black. Empty unless the palette binds both to colors.
fn unlit_material_ids(main: &VoxMain<()>, palette: &VoxPalette) -> HashSet<U32Id<BVoxMaterial>> {
    let colors = |name: &str| {
        let property_id = palette.property_id_by_name(name)?;
        let property = palette.property(property_id).expect("a named property");
        let colors = ColorValues::of(
            main.value_pool(property.value_pool_id)
                .expect("a property draws from a live value pool"),
        )?;
        Some((property_id, colors))
    };
    let (Some(emissive), Some(base)) = (colors(EMISSIVE_COLOR), colors(BASE_COLOR)) else {
        return HashSet::new();
    };
    let black = |(property_id, colors): &(U32Id<BVoxProperty>, ColorValues), material_id| {
        let value_id = palette
            .value_id(material_id, *property_id)
            .expect("a live material has a value id for every property");
        let color = colors
            .lin_srgba_f64(value_id)
            .expect("a live material draws one of its property's values");
        color.red == 0.0 && color.green == 0.0 && color.blue == 0.0
    };

    palette
        .iter_materials()
        .filter(|&material_id| black(&emissive, material_id) && !black(&base, material_id))
        .collect()
}

/// Whether `palette`'s material axis, its `(property, value pool)` pairs,
/// already holds one value per slot the way the writer reads it: every value
/// pool as long as the first, numbered densely from zero and no longer than
/// [`MATERIAL_SLOTS`], and every material drawing one value id from them all.
fn in_slots(
    main: &VoxMain<()>,
    palette: &VoxPalette,
    material_axis: &[(U32Id<BVoxProperty>, U32Id<BVoxValuePool>)],
) -> bool {
    let value_pool = |value_pool_id| {
        main.value_pool(value_pool_id)
            .expect("a property draws from a live value pool")
    };
    let Some(&(_, first_value_pool_id)) = material_axis.first() else {
        return true;
    };
    let slot_count = value_pool(first_value_pool_id).len();
    let dense = material_axis.iter().all(|&(_, value_pool_id)| {
        let value_pool = value_pool(value_pool_id);
        value_pool.len() == slot_count
            && IdRange::from_len(slot_count).all(|value_id| value_pool.contains_value(value_id))
    });

    slot_count <= MATERIAL_SLOTS
        && dense
        && palette.iter_materials().all(|material_id| {
            let mut value_ids = material_axis.iter().map(|&(property_id, _)| {
                palette
                    .value_id(material_id, property_id)
                    .expect("a live material has a value id for every property")
            });
            let first = value_ids.next();
            value_ids.all(|value_id| Some(value_id) == first)
        })
}

/// Whether value ids `a` and `b` of `value_pool` hold equal values.
fn same_value(
    value_pool: &VoxValuePool,
    a: U32Id<BVoxValuePoolValue>,
    b: U32Id<BVoxValuePoolValue>,
) -> bool {
    fn equal<T: PartialEq>(
        values: VoxValueColumn<'_, T>,
        a: U32Id<BVoxValuePoolValue>,
        b: U32Id<BVoxValuePoolValue>,
    ) -> bool {
        values.get(a) == values.get(b)
    }

    match value_pool.values() {
        VoxValuePoolValues::Bool(values) => equal(values, a, b),
        VoxValuePoolValues::Float(values) => equal(values, a, b),
        VoxValuePoolValues::Int(values) => equal(values, a, b),
        VoxValuePoolValues::Json(values) => equal(values, a, b),
        VoxValuePoolValues::String(values) => equal(values, a, b),
        VoxValuePoolValues::Vec2Float(values) => equal(values, a, b),
        VoxValuePoolValues::Vec2Int(values) => equal(values, a, b),
        VoxValuePoolValues::Vec3Float(values) => equal(values, a, b),
        VoxValuePoolValues::Vec3Int(values) => equal(values, a, b),
        VoxValuePoolValues::Vec4Float(values) => equal(values, a, b),
        VoxValuePoolValues::Vec4Int(values) => equal(values, a, b),
    }
}

/// A value pool of `value_pool`'s kind holding its values at `value_ids`, in
/// that order, repeats included.
fn gathered_value_pool(
    value_pool: &VoxValuePool,
    value_ids: &[U32Id<BVoxValuePoolValue>],
) -> Result<VoxValuePool> {
    fn gathered<T: Clone>(
        values: VoxValueColumn<'_, T>,
        value_ids: &[U32Id<BVoxValuePoolValue>],
    ) -> Vec<T> {
        value_ids
            .iter()
            .map(|&value_id| {
                values
                    .get(value_id)
                    .expect("a material draws one of its property's values")
                    .clone()
            })
            .collect()
    }

    Ok(match value_pool.values() {
        VoxValuePoolValues::Bool(values) => VoxValuePool::boolean(gathered(values, value_ids)),
        VoxValuePoolValues::Float(values) => VoxValuePool::float(gathered(values, value_ids))?,
        VoxValuePoolValues::Int(values) => VoxValuePool::int(gathered(values, value_ids))?,
        VoxValuePoolValues::Json(values) => VoxValuePool::json(gathered(values, value_ids)),
        VoxValuePoolValues::String(values) => VoxValuePool::string(gathered(values, value_ids)),
        VoxValuePoolValues::Vec2Float(values) => {
            VoxValuePool::vec_2_float(gathered(values, value_ids))?
        }
        VoxValuePoolValues::Vec2Int(values) => {
            VoxValuePool::vec_2_int(gathered(values, value_ids))?
        }
        VoxValuePoolValues::Vec3Float(values) => {
            VoxValuePool::vec_3_float(gathered(values, value_ids))?
        }
        VoxValuePoolValues::Vec3Int(values) => {
            VoxValuePool::vec_3_int(gathered(values, value_ids))?
        }
        VoxValuePoolValues::Vec4Float(values) => {
            VoxValuePool::vec_4_float(gathered(values, value_ids))?
        }
        VoxValuePoolValues::Vec4Int(values) => {
            VoxValuePool::vec_4_int(gathered(values, value_ids))?
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        FALLBACK_CONTENT_VERSION, SHADOWS, SYNTH_CAMERA, SceneCameraSource, VMaxColorFormat,
        VMaxExtPalette, VMaxWriteOptions, decode_axis_angle, from_vmax_file, to_vmax_file,
        to_vmax_vox_main,
    };
    use branded_id::{IdRange, U32Id};
    use std::collections::{BTreeMap, BTreeSet, HashSet};
    use ty_math::{
        TyHexColor, TyQuaternionF64, TySrgbaU8, TyTransformF64, TyVector3Ext, TyVector3F64,
        TyVector3U32,
    };
    use vmax::{
        VMaxFile, VMaxSceneCamera,
        snapshots::{VMaxVoxel, decode_vmax_snapshots},
    };
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, BVoxValuePool,
        BVoxValuePoolValue, BVoxVoxel, VoxEffectivePalette, VoxExt, VoxHierarchyNode, VoxMain,
        VoxObject, VoxPalette, VoxValuePool,
        color::{ColorValues, lin_srgba_f64_from_srgba_u8},
        material::{BASE_COLOR, EMISSIVE_COLOR, EMISSIVE_STRENGTH, METALLIC, ROUGHNESS},
    };

    fn options(color_format: VMaxColorFormat) -> VMaxWriteOptions {
        VMaxWriteOptions {
            color_format,
            ..Default::default()
        }
    }

    #[test]
    fn a_non_color_base_color_errors_rather_than_writing_a_blank_palette() {
        // A transparent stand-in would write a model Voxel Max renders as
        // entirely invisible, at exit 0.
        let mut main = VoxMain::default();
        let value_pool_id = main.retain_value_pool(VoxValuePool::float(vec![0.5]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id)
            .unwrap();
        let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id).unwrap();
        object
            .retain_voxel(U32Id::from_u32(0), &[material_id])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![object_id],
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &options(VMaxColorFormat::All),
        )
        .unwrap_err();
        assert!(error.to_string().contains(BASE_COLOR), "{error}");
    }

    /// A `shadows` over a pool of another kind errors rather than writing the
    /// shadow-casting default.
    #[test]
    fn a_non_flag_shadows_errors_rather_than_writing_the_default() {
        let mut main = VoxMain::default();
        let colors_id = main
            .retain_value_pool(VoxValuePool::vec_4_float(vec![color_floats("#FF0000FF")]).unwrap());

        let floats_id = main.retain_value_pool(VoxValuePool::float(vec![0.0]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors_id)
            .unwrap();

        palette
            .retain_property(SHADOWS.to_owned(), floats_id)
            .unwrap();

        let material_id = palette
            .retain_material(vec![U32Id::from_u32(0), U32Id::from_u32(0)])
            .unwrap();

        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id).unwrap();
        object
            .retain_voxel(U32Id::from_u32(0), &[material_id])
            .unwrap();

        main.retain_object(object).unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();

        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap_err();

        assert!(error.to_string().contains("holding no flag"), "{error}");
    }

    /// A material pool holding a value past the eight slots is no Voxel Max
    /// palette, so the writer errors rather than wrapping the slot.
    #[test]
    fn a_material_pool_past_the_slots_errors() {
        let mut main = VoxMain::default();
        let color_value_pool_id = main
            .retain_value_pool(VoxValuePool::vec_4_float(vec![color_floats("#FF0000FF")]).unwrap());
        // Nine metallic values fill nine slots, one past the eight; the color
        // value pool stays a single in-range color.
        let metallic_value_pool_id = main.retain_value_pool(
            VoxValuePool::float((0..9u32).map(|index| f64::from(index) / 10.0).collect()).unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property("baseColor".to_owned(), color_value_pool_id)
            .unwrap();
        palette
            .retain_property("metallic".to_owned(), metallic_value_pool_id)
            .unwrap();
        let material_ids: Vec<_> = IdRange::from_len(9)
            .map(|value_id| {
                palette
                    .retain_material(vec![U32Id::from_u32(0), value_id])
                    .unwrap()
            })
            .collect();
        let palette_id = main.retain_palette(palette).unwrap();
        let mut object = VoxObject::new(String::new(), TyVector3U32::new(9, 1, 1)).unwrap();
        object.retain_layer(palette_id).unwrap();
        for (x, &material_id) in (0..).zip(&material_ids) {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        main.retain_object(object).unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("material slots"), "{error}");
    }

    /// A Voxel Max object reads one palette and the writer converts nothing,
    /// so an object layering a second palette errors.
    #[test]
    fn a_second_layer_errors() {
        let mut main = VoxMain::default();
        let color_palette_id =
            retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF", "#0000FFFF"]);
        let metallic_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![0.0, 1.0]).unwrap());
        let mut material_palette = VoxPalette::default();
        material_palette
            .retain_property("metallic".to_owned(), metallic_value_pool_id)
            .unwrap();
        for value_id in IdRange::from_len(2) {
            material_palette.retain_material(vec![value_id]).unwrap();
        }
        let material_palette_id = main.retain_palette(material_palette).unwrap();

        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);
        let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(color_palette_id).unwrap();
        object.retain_layer(material_palette_id).unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[material_id(1), material_id(1)])
            .unwrap();
        main.retain_object(object).unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("layers"), "{error}");
    }

    /// Voxel Max keeps a material list only in a color palette's sidecar, so a
    /// palette binding materials but no `baseColor` errors.
    #[test]
    fn materials_without_a_base_color_error() {
        let mut main = VoxMain::default();
        let metallic_value_pool_id =
            main.retain_value_pool(VoxValuePool::float(vec![0.5]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("metallic".to_owned(), metallic_value_pool_id)
            .unwrap();
        palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();
        let palette_id = main.retain_palette(palette).unwrap();
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains(BASE_COLOR), "{error}");
    }

    /// A state carrying no format ext, built straight from voxcore: a red-green
    /// object and a blue object sharing one `rgba` palette, placed by a small
    /// hierarchy of a nested group and two roots. The writer must synthesize
    /// the `vmax` ext from this rather than read one. Red is cell 0, so a
    /// live voxel references the palette index Voxel Max reads as empty;
    /// synthesis must shift it past index 0 rather than drop the voxel.
    fn source_state() -> VoxMain<()> {
        let mut main = VoxMain::default();

        // One palette whose first real color is material 0: red, green, blue.
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF", "#0000FFFF"]);
        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red (material 0) then a green (material 1) voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.retain_layer(palette_id).unwrap();
        for (x, material_index) in [(0u32, 0u32), (1, 1)] {
            let voxel_id = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel_id, &[material_id(material_index)])
                .expect("one sample for the one layer");
        }
        main.retain_object(wide).unwrap();

        // Object 1: a single blue (material 2) voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.retain_layer(palette_id).unwrap();
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(2)])
            .expect("one sample for the one layer");
        main.retain_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at = |x: f64, y: f64, z: f64| {
            TyTransformF64::new(
                TyVector3F64::new(x, y, z),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::new(1.0, 1.0, 1.0),
            )
        };

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        main.retain_hierarchy_nodes(vec![
            VoxHierarchyNode {
                name: "group".to_owned(),
                child_node_ids: vec![node_id(1)],
                child_object_ids: Vec::new(),
                transform: TyTransformF64::default(),
            },
            VoxHierarchyNode {
                name: "wide".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id(0)],
                transform: placed_at(5.0, 0.0, 0.0),
            },
            VoxHierarchyNode {
                name: "unit".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id(1)],
                transform: placed_at(0.0, 3.0, 0.0),
            },
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        main.validate().expect("a well-formed source");
        main
    }

    /// Every live voxel as `(world position, color)`, walking the hierarchy
    /// from the roots and accumulating each node's translation.
    /// Order-independent and resolved per voxel, so a state compares to one
    /// round-tripped through a synthesized document without depending on
    /// object, palette, or voxel order.
    fn world_voxels<T: VoxExt>(main: &VoxMain<T>) -> BTreeSet<([i32; 3], [u8; 4])> {
        fn walk<T: VoxExt>(
            main: &VoxMain<T>,
            node_id: U32Id<BVoxHierarchyNode>,
            origin: [i32; 3],
            voxels: &mut BTreeSet<([i32; 3], [u8; 4])>,
        ) {
            let node = main.hierarchy_node(node_id).expect("a valid node");
            let position = node.transform.position;
            let translation = [
                origin[0] + position.x.round() as i32,
                origin[1] + position.y.round() as i32,
                origin[2] + position.z.round() as i32,
            ];
            for &object_id in &node.child_object_ids {
                let object = main.object(object_id).expect("a valid object");
                let effective = main
                    .effective_palette(object)
                    .expect("the test layers resolve");
                // A voxel sits at node-local `origin + grid`; these states use
                // identity rotation and unit scale, so the world position is
                // the accumulated translation plus that offset.
                let object_origin = object.origin();
                for voxel_id in object.iter_live() {
                    let grid = object.voxel_position(voxel_id).expect("within the grid");
                    let world = [
                        translation[0] + object_origin.x + grid.x as i32,
                        translation[1] + object_origin.y + grid.y as i32,
                        translation[2] + object_origin.z + grid.z as i32,
                    ];
                    let rgba = cell_color(&effective, object, voxel_id);
                    voxels.insert((world, rgba));
                }
            }
            for &child_id in &node.child_node_ids {
                walk(main, child_id, translation, voxels);
            }
        }

        let mut voxels = BTreeSet::new();
        for &root_id in main.root_hierarchy_node_ids() {
            walk(main, root_id, [0, 0, 0], &mut voxels);
        }
        voxels
    }

    /// The color of the live voxel at `voxel_id`, read through the effective
    /// palette's `baseColor` at the material the voxel samples in the winning
    /// layer, or transparent black when no layer supplies one.
    fn cell_color(
        effective: &VoxEffectivePalette<'_>,
        object: &VoxObject,
        voxel_id: U32Id<BVoxVoxel>,
    ) -> [u8; 4] {
        let Some(property_id) = effective.property_id_by_name(BASE_COLOR) else {
            return [0, 0, 0, 0];
        };
        let property = effective
            .property(property_id)
            .expect("a resolved name identifies one of the effective palette's properties");
        let material_id = object
            .voxel_material(voxel_id, property.layer_id())
            .expect("a live voxel samples the winning layer");
        let value_id = property
            .value_id(material_id)
            .expect("a material has a value id for every property");
        ColorValues::of(property.value_pool())
            .and_then(|colors| colors.srgba_u8(value_id))
            .expect("the test colors resolve")
    }

    /// A default state has no scene, so the writer synthesizes an empty
    /// document rather than erroring on the missing ext.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let main = VoxMain::default();
        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        assert!(file.scene_json_file.objects.is_empty());
        assert!(file.scene_json_file.groups.is_empty());
    }

    /// A state with no `vmax` ext, such as one cross-loaded from another
    /// format, synthesizes a document that `from_vmax_file` reads back with the
    /// same world geometry, colors, and placement.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let source = source_state();
        let expected = world_voxels(&source);
        let file = to_vmax_file(
            &to_vmax_vox_main(source).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(world_voxels(&reloaded), expected);
    }

    /// Synthesis gives every node a distinct `ind`, since Voxel Max collapses
    /// nodes that share the `[0, 0, 0]` path triple onto one. Groups take the
    /// `1` lane and objects the `0` lane.
    #[test]
    fn synthesizes_distinct_node_ind() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        // A root group parenting two object nodes.
        main.retain_hierarchy_nodes(vec![
            group_node("g", &[1, 2], at(0.0, 0.0, 0.0)),
            object_node("a", 0, at(0.0, 0.0, 0.0)),
            object_node("b", 1, at(5.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
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

    /// A node placing several objects, as a Goxel layer's blocks do, keeps
    /// every object: Voxel Max models one object per node, so synthesis
    /// flattens the extra objects to siblings that share the node's placement
    /// rather than dropping all but the first.
    #[test]
    fn synthesizes_a_node_placing_several_objects() {
        let mut main = VoxMain::default();

        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF"]);
        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Two unit objects, a red one (material 0) and a green one (material 1).
        for material_index in [0u32, 1] {
            let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
                .expect("a 1x1x1 grid is within the dense limit");
            object.retain_layer(palette_id).unwrap();
            let voxel_id = object
                .voxel_id(TyVector3U32::new(0, 0, 0))
                .expect("a position within the grid");
            object
                .retain_voxel(voxel_id, &[material_id(material_index)])
                .expect("one sample for the one layer");
            main.retain_object(object).unwrap();
        }

        // One node placing both objects at the same offset, so they coincide in
        // the world at distinct colors.
        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        main.retain_hierarchy_node(VoxHierarchyNode {
            name: "layer".to_owned(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![object_id(0), object_id(1)],
            transform: TyTransformF64::new(
                TyVector3F64::new(10.0, 0.0, 0.0),
                TyQuaternionF64::IDENTITY,
                TyVector3F64::new(1.0, 1.0, 1.0),
            ),
        })
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id(0)]).unwrap();
        main.validate().expect("a well-formed source");

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 2);
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([10, 0, 0], red), ([10, 0, 0], green)])
        );
    }

    /// The linear-light `[f64; 4]` components of a `#RRGGBB` or `#RRGGBBAA`
    /// color. A missing alpha defaults to opaque.
    fn color_floats(hex: &str) -> [f64; 4] {
        lin_srgba_f64_from_srgba_u8(TySrgbaU8::parse_hex(hex).expect("a valid hex color")).into()
    }

    /// Adds a palette binding `baseColor` to a value pool of the
    /// given colors, with one material per color so a material's index is its
    /// color index, and returns the palette id.
    fn retain_rgba_palette(main: &mut VoxMain<()>, hexes: &[&str]) -> U32Id<BVoxPalette> {
        let value_pool_id = main.retain_value_pool(
            VoxValuePool::vec_4_float(hexes.iter().map(|hex| color_floats(hex)).collect()).unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property("baseColor".to_owned(), value_pool_id)
            .unwrap();
        for value_id in IdRange::from_len(hexes.len()) {
            palette
                .retain_material(vec![value_id])
                .expect("one value id per property");
        }
        main.retain_palette(palette).unwrap()
    }

    /// An object of `bounds` whose live voxels each sample one material, given as
    /// `(position, material)` pairs.
    fn color_object(
        palette_id: U32Id<BVoxPalette>,
        bounds: TyVector3U32,
        voxels: &[([u32; 3], u32)],
    ) -> VoxObject {
        let mut object = VoxObject::new(String::new(), bounds).expect("within the dense limit");
        object.retain_layer(palette_id).unwrap();
        for &([x, y, z], material_index) in voxels {
            let voxel_id = object
                .voxel_id(TyVector3U32::new(x, y, z))
                .expect("a position within the grid");
            object
                .retain_voxel(voxel_id, &[U32Id::<BVoxMaterial>::from_u32(material_index)])
                .expect("one sample for the one layer");
        }
        object
    }

    /// A translation-only transform.
    fn at(x: f64, y: f64, z: f64) -> TyTransformF64 {
        TyTransformF64::new(
            TyVector3F64::new(x, y, z),
            TyQuaternionF64::IDENTITY,
            TyVector3F64::new(1.0, 1.0, 1.0),
        )
    }

    /// A node placing object `object_id` at `transform`.
    fn object_node(name: &str, object_id: u32, transform: TyTransformF64) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![U32Id::<BVoxObject>::from_u32(object_id)],
            transform,
        }
    }

    /// A group node parenting the given child nodes at `transform`.
    fn group_node(name: &str, child_ids: &[u32], transform: TyTransformF64) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids: child_ids
                .iter()
                .map(|&child_id| U32Id::<BVoxHierarchyNode>::from_u32(child_id))
                .collect(),
            child_object_ids: Vec::new(),
            transform,
        }
    }

    /// The voxels decoded from a contents file's snapshots.
    fn contents_voxels(file: &VMaxFile, name: &str) -> Vec<VMaxVoxel> {
        decode_vmax_snapshots(&file.contents_files[name].snapshots).expect("decodable snapshots")
    }

    /// The image stores colors 0-based then a transparent terminator, and
    /// voxels take 1-based indices: the first color sits at image index 0, the
    /// terminator at 255, and each voxel's color_idx is its cell + 1.
    #[test]
    fn synthesizes_colors_zero_based_with_a_terminator() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let png = &file.palette_png_files["palette1.png"].0;
        assert_eq!(png.len(), 256);
        assert_eq!(png[0], [0xFF, 0, 0, 0xFF]);
        assert_eq!(png[1], [0, 0xFF, 0, 0xFF]);
        assert_eq!(png[255], [0, 0, 0, 0]);
        let mut indices: Vec<u8> = contents_voxels(&file, "contents.vmaxb")
            .iter()
            .map(|voxel| voxel.color_idx)
            .collect();
        indices.sort_unstable();
        assert_eq!(indices, [1, 2]);
    }

    /// A voxel on the first color (index 1) and one on the last (index 255)
    /// round-trip through a plist palette, whose `colors` table is 0-based.
    /// Guards the bug a real Voxel Max plist file hit: the last color read back
    /// as out of range.
    #[test]
    fn round_trips_first_and_last_color_through_plist() {
        let mut main = VoxMain::default();
        // 255 colors: red first, blue last, green between.
        let mut hexes = vec!["#FF0000FF".to_owned()];
        hexes.extend((0..253).map(|_| "#00FF00FF".to_owned()));
        hexes.push("#0000FFFF".to_owned());
        let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let palette_id = retain_rgba_palette(&mut main, &refs);
        // A voxel on the first color (cell 0) and one on the last (cell 254).
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 254)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &options(VMaxColorFormat::Plist),
        )
        .unwrap();
        // The plist colors are 0-based: red first, blue last.
        let colors: Vec<[u8; 4]> = file.palette_settings_files["palette1.settings.vmaxpsb"]
            .colors
            .as_chunks::<4>()
            .0
            .to_vec();
        assert_eq!(colors.len(), 255);
        assert_eq!(colors[0], [0xFF, 0, 0, 0xFF]);
        assert_eq!(colors[254], [0, 0, 0xFF, 0xFF]);
        // Voxels carry 1-based indices: the first color is 1, the last is 255.
        let mut indices: Vec<u8> = contents_voxels(&file, "contents.vmaxb")
            .iter()
            .map(|voxel| voxel.color_idx)
            .collect();
        indices.sort_unstable();
        assert_eq!(indices, [1, 255]);
        // It reads back without the out-of-range error, resolving to red and
        // blue.
        let reloaded = from_vmax_file(&file).unwrap();
        let resolved: BTreeSet<[u8; 4]> = world_voxels(&reloaded)
            .into_iter()
            .map(|(_, rgba)| rgba)
            .collect();
        assert_eq!(
            resolved,
            BTreeSet::from([[0xFF, 0, 0, 0xFF], [0, 0, 0xFF, 0xFF]])
        );
    }

    /// A palette with more colors than fit is rejected rather than silently
    /// truncated or wrapped: 255 colors is the budget and round-trips, 256
    /// overflows.
    #[test]
    fn errors_when_colors_exceed_palette_budget() {
        let synthesize = |count: u32| {
            let mut main = VoxMain::default();
            let hexes: Vec<String> = (0..count).map(|i| format!("#{:06X}FF", i)).collect();
            let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
            let palette_id = retain_rgba_palette(&mut main, &refs);
            let voxels: Vec<([u32; 3], u32)> = (0..count).map(|i| ([i, 0, 0], i)).collect();
            main.retain_object(color_object(
                palette_id,
                TyVector3U32::new(count, 1, 1),
                &voxels,
            ))
            .unwrap();
            main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
                .unwrap();
            main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
                .unwrap();
            main.validate().unwrap();
            to_vmax_file(
                &to_vmax_vox_main(main).unwrap(),
                &VMaxWriteOptions::default(),
            )
        };

        let file = synthesize(255).expect("255 colors fit the palette");
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            reloaded.object(U32Id::from_u32(0)).unwrap().live_count(),
            255
        );
        assert!(synthesize(256).is_err());
    }

    /// An object with no color palette borrows the default palette name rather
    /// than an empty `pal` Voxel Max cannot resolve, and writes no file of its
    /// own. The colored and empty objects share that name.
    #[test]
    fn synthesizes_a_colorless_object_sharing_the_default_palette() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        // Object 0: a single red voxel. Object 1: an empty, colorless object,
        // whose tight grid is [0, 0, 0].
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_object(VoxObject::new(String::new(), TyVector3U32::new(0, 0, 0)).unwrap())
            .unwrap();
        main.retain_hierarchy_nodes(vec![
            object_node("colored", 0, at(0.0, 0.0, 0.0)),
            object_node("colorless", 1, at(10.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        // Both objects name the one real palette; no extra or placeholder file.
        let names: BTreeSet<&str> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.palette.as_str())
            .collect();
        assert_eq!(names, BTreeSet::from(["palette1.png"]));
        assert_eq!(
            file.palette_png_files.keys().collect::<Vec<_>>(),
            ["palette1.png"]
        );

        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(world_voxels(&reloaded), BTreeSet::from([([0, 0, 0], red)]));
    }

    /// A colorless object's live voxels take a non-empty index (the borrowed
    /// palette's first color), not the empty index 0, so they do not vanish.
    #[test]
    fn synthesizes_colorless_voxels_on_a_non_empty_index() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        // Object 0: a colored voxel, so a `palette.png` exists to borrow.
        // Object 1: a colorless object that nonetheless has a live voxel.
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        let mut colorless = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
        let voxel_id = colorless.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        colorless.retain_voxel(voxel_id, &[]).unwrap();
        main.retain_object(colorless).unwrap();
        main.retain_hierarchy_nodes(vec![
            object_node("colored", 0, at(0.0, 0.0, 0.0)),
            object_node("colorless", 1, at(10.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        // No live voxel, colored or colorless, lands on the empty index 0.
        let indices: Vec<u8> = file
            .contents_files
            .values()
            .flat_map(|contents| decode_vmax_snapshots(&contents.snapshots).unwrap())
            .map(|voxel| voxel.color_idx)
            .collect();
        assert!(!indices.is_empty());
        assert!(indices.iter().all(|&index| index >= 1));
    }

    /// A node placing several objects and also parenting child nodes becomes a
    /// group, since Voxel Max objects are leaves: its objects flatten to
    /// sibling objects sharing the node's placement, the child nodes sit beside
    /// them under the group, and every object gets its own files.
    #[test]
    fn synthesizes_a_node_placing_objects_and_child_nodes() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF", "#0000FFFF"]);
        for cell in 0..3 {
            main.retain_object(color_object(
                palette_id,
                TyVector3U32::new(1, 1, 1),
                &[([0, 0, 0], cell)],
            ))
            .unwrap();
        }
        // A fourth object placed by a child node, which sits beside the three
        // under the group.
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        // node 0 places objects 0, 1, 2 at +10x and parents node 1; node 1
        // places object 3 at +1y of node 0.
        main.retain_hierarchy_nodes(vec![
            VoxHierarchyNode {
                name: "layer".to_owned(),
                child_node_ids: vec![U32Id::<BVoxHierarchyNode>::from_u32(1)],
                child_object_ids: vec![
                    U32Id::<BVoxObject>::from_u32(0),
                    U32Id::<BVoxObject>::from_u32(1),
                    U32Id::<BVoxObject>::from_u32(2),
                ],
                transform: at(10.0, 0.0, 0.0),
            },
            object_node("child", 3, at(0.0, 1.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        // Four scene objects: three from the multi-object node, one from the
        // child.
        assert_eq!(file.scene_json_file.objects.len(), 4);
        // The three siblings share the parent's id, and ids are all distinct.
        let ids: BTreeSet<&str> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.id.as_str())
            .collect();
        assert_eq!(ids.len(), 4);
        // Every object sits in a group, the only parent Voxel Max resolves.
        let group_ids: BTreeSet<&str> = file
            .scene_json_file
            .groups
            .iter()
            .map(|group| group.id.as_str())
            .collect();
        assert!(file.scene_json_file.objects.iter().all(|object| {
            object
                .parent_id
                .as_deref()
                .is_some_and(|parent_id| group_ids.contains(parent_id))
        }));

        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        let blue = [0, 0, 0xFF, 0xFF];
        // The three siblings keep their place, and the child keeps its place
        // +1y of the node.
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

    /// Two nodes placing one object instance it: the object's voxels are
    /// rebuilt once into a shared contents file, the scene carries two objects
    /// with distinct ids, and reloading collapses them back to one voxcore
    /// object placed at both positions.
    #[test]
    fn synthesizes_instances_sharing_one_contents_file() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_hierarchy_nodes(vec![
            object_node("a", 0, at(0.0, 0.0, 0.0)),
            object_node("b", 0, at(20.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
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
    /// duplicates it per path so the rebuilt world matches voxcore, where a
    /// node's placement composes along every path to it.
    #[test]
    fn synthesizes_a_shared_subtree_at_every_parent() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        // node 2 places object 0 at +1x and is a child of both group node 0 (at
        // the origin) and group node 1 (at +100x).
        main.retain_hierarchy_nodes(vec![
            group_node("a", &[2], at(0.0, 0.0, 0.0)),
            group_node("b", &[2], at(100.0, 0.0, 0.0)),
            object_node("c", 0, at(1.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
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
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        // node 1 places object 0 at +1x; it is both a root and a child of group
        // node 0 at +100x.
        main.retain_hierarchy_nodes(vec![
            group_node("a", &[1], at(100.0, 0.0, 0.0)),
            object_node("c", 0, at(1.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([1, 0, 0], red), ([101, 0, 0], red)])
        );
    }

    /// A node reachable from no root is dropped, since voxcore never places it,
    /// so synthesis does not promote it to a spurious root.
    #[test]
    fn drops_a_node_unreachable_from_the_roots() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        // node 0 is a root placing object 0; node 1 places object 1 but is
        // neither a root nor anyone's child.
        main.retain_hierarchy_nodes(vec![
            object_node("rooted", 0, at(0.0, 0.0, 0.0)),
            object_node("orphan", 1, at(50.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 1);
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        assert_eq!(world_voxels(&reloaded), BTreeSet::from([([0, 0, 0], red)]));
    }

    /// Synthesis encodes the node's quaternion rotation back to `t_r`, so
    /// rotation survives alongside translation and scale; the lone voxel still
    /// renders at its original world point.
    #[test]
    fn keeps_rotation_scale_and_translation() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        let rotation =
            TyQuaternionF64::from_axis_angle(TyVector3F64::new(0.0, 0.0, 1.0), 90f64.to_radians());
        let transform = TyTransformF64::new(
            TyVector3F64::new(7.0, 0.0, 0.0),
            rotation,
            TyVector3F64::new(2.0, 1.0, 1.0),
        );
        main.retain_hierarchy_node(object_node("o", 0, transform))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let node = reloaded.hierarchy_node(U32Id::from_u32(0)).unwrap();

        // The axis-angle round-trip reproduces the rotation to floating-point
        // noise.
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        let r = node.transform.rotation;
        assert!(close(r.x, rotation.x) && close(r.y, rotation.y));
        assert!(close(r.z, rotation.z) && close(r.w, rotation.w));
        assert_eq!(node.transform.scale, TyVector3F64::new(2.0, 1.0, 1.0));

        // node.position is the content-center pivot and the rotation now
        // applies to the grid, so the lone voxel still renders at its original
        // world point: position + R * (scale * (origin + local)), with local at
        // the grid corner.
        let object = reloaded.object(U32Id::from_u32(0)).unwrap();
        let origin = object.origin();
        let s = node.transform.scale;
        let local = TyVector3F64::new(
            s.x * origin.x as f64,
            s.y * origin.y as f64,
            s.z * origin.z as f64,
        );
        let world = node.transform.position + r * local;
        assert!(close(world.x, 7.0) && close(world.y, 0.0) && close(world.z, 0.0));
    }

    /// A material palette survives synthesis across every color format, and the
    /// material index is not shifted by the color offset.
    #[test]
    fn synthesizes_material_palettes_across_color_formats() {
        let formats = [
            VMaxColorFormat::Png,
            VMaxColorFormat::Plist,
            VMaxColorFormat::All,
        ];
        for format in formats {
            let mut main = VoxMain::default();
            // One palette binding a color plus the material attributes, with a
            // single material glowing in its own color. No ext, so the writer
            // reads the Voxel Max material from the value pools.
            let red = color_floats("#FF0000FF");
            let color = main.retain_value_pool(VoxValuePool::vec_4_float(vec![red]).unwrap());
            let emissive_color = main.retain_value_pool(
                VoxValuePool::vec_3_float(vec![[red[0], red[1], red[2]]]).unwrap(),
            );
            let float = |value: f64| VoxValuePool::float(vec![value]).unwrap();
            let metallic = main.retain_value_pool(float(0.5));
            let roughness = main.retain_value_pool(float(0.25));
            let emissive = main.retain_value_pool(float(2.0));
            let shadows = main.retain_value_pool(VoxValuePool::boolean(vec![true]));
            let mut palette = VoxPalette::default();
            palette
                .retain_property("baseColor".to_owned(), color)
                .unwrap();
            palette
                .retain_property("metallic".to_owned(), metallic)
                .unwrap();
            palette
                .retain_property("roughness".to_owned(), roughness)
                .unwrap();
            palette
                .retain_property("emissiveColor".to_owned(), emissive_color)
                .unwrap();
            palette
                .retain_property("emissiveStrength".to_owned(), emissive)
                .unwrap();
            palette
                .retain_property("shadows".to_owned(), shadows)
                .unwrap();
            palette
                .retain_material(vec![U32Id::from_u32(0); 6])
                .unwrap();
            let palette_id = main.retain_palette(palette).unwrap();
            let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
            object.retain_layer(palette_id).unwrap();
            let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
            object
                .retain_voxel(voxel_id, &[U32Id::<BVoxMaterial>::from_u32(0)])
                .unwrap();
            main.retain_object(object).unwrap();
            main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
                .unwrap();
            main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
                .unwrap();
            main.validate().unwrap();

            let file = to_vmax_file(&to_vmax_vox_main(main).unwrap(), &options(format)).unwrap();
            // The material byte is 0-based and independent of the color offset:
            // the one material is index 0.
            assert!(
                contents_voxels(&file, "contents.vmaxb")
                    .iter()
                    .all(|voxel| voxel.material_idx == 0)
            );

            let reloaded = from_vmax_file(&file).unwrap();
            let (palette_id, material_palette) = reloaded
                .iter_palettes()
                .find(|(_, palette)| palette.property_id_by_name("metallic").is_some())
                .expect("a material palette survives");
            let material_id = material_palette
                .iter_materials()
                .next()
                .expect("one material");
            let scalar = |attribute: &str| -> f64 {
                let property_id = material_palette.property_id_by_name(attribute).unwrap();
                let (value_pool, value_id) = reloaded
                    .material_value(palette_id, material_id, property_id)
                    .unwrap();
                *value_pool
                    .float_values()
                    .expect("a scalar attribute is a float value pool")
                    .get(value_id)
                    .unwrap()
            };
            let flag = |attribute: &str| -> bool {
                let property_id = material_palette.property_id_by_name(attribute).unwrap();
                let (value_pool, value_id) = reloaded
                    .material_value(palette_id, material_id, property_id)
                    .unwrap();
                *value_pool
                    .boolean_values()
                    .expect("shadows is a bool value pool")
                    .get(value_id)
                    .unwrap()
            };
            // Metalness and roughness round-trip through Voxel Max's 0.1 to 0.9
            // coefficient range, so they return within f64 rounding of the
            // linear map rather than bit-exact. Emissive stays a raw scalar.
            let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
            assert!(close(scalar("metallic"), 0.5));
            assert!(close(scalar("roughness"), 0.25));
            assert_eq!(scalar("emissiveStrength"), 2.0);
            assert!(flag("shadows"));
        }
    }

    /// A one-voxel state binding `baseColor`, `emissiveColor`, and
    /// `emissiveStrength` on one material, for the emissive tests.
    fn emissive_main(base: [f64; 4], emissive: [f64; 3], strength: f64) -> VoxMain<()> {
        let mut main = VoxMain::default();
        let color = main.retain_value_pool(VoxValuePool::vec_4_float(vec![base]).unwrap());
        let emissive_color =
            main.retain_value_pool(VoxValuePool::vec_3_float(vec![emissive]).unwrap());
        let strength = main.retain_value_pool(VoxValuePool::float(vec![strength]).unwrap());
        let mut palette = VoxPalette::default();
        palette
            .retain_property("baseColor".to_owned(), color)
            .unwrap();
        palette
            .retain_property("emissiveColor".to_owned(), emissive_color)
            .unwrap();
        palette
            .retain_property("emissiveStrength".to_owned(), strength)
            .unwrap();
        palette
            .retain_material(vec![U32Id::from_u32(0); 3])
            .unwrap();
        let palette_id = main.retain_palette(palette).unwrap();
        let mut object = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(palette_id).unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::<BVoxMaterial>::from_u32(0)])
            .unwrap();
        main.retain_object(object).unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();
        main
    }

    /// The `sic` the written document carries for `main`'s one material, read
    /// back through from-vmax, which carries it as `emissiveStrength`.
    fn written_sic(main: VoxMain<()>) -> f64 {
        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &options(VMaxColorFormat::All),
        )
        .unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let (palette_id, material_palette) = reloaded
            .iter_palettes()
            .find(|(_, palette)| palette.property_id_by_name("emissiveStrength").is_some())
            .expect("a material palette survives");
        let material_id = material_palette.iter_materials().next().unwrap();
        let property_id = material_palette
            .property_id_by_name("emissiveStrength")
            .unwrap();
        let (value_pool, value_id) = reloaded
            .material_value(palette_id, material_id, property_id)
            .unwrap();
        *value_pool
            .float_values()
            .expect("emissiveStrength is a float value pool")
            .get(value_id)
            .unwrap()
    }

    /// Voxel Max glows in the voxel's base color at coefficient `sic`, so an
    /// emissive color equal to the base color writes the strength exactly,
    /// left unbounded.
    #[test]
    fn writes_a_matching_emissive_as_the_strength() {
        let base = color_floats("#808080FF");
        let main = emissive_main(base, [base[0], base[1], base[2]], 2.0);
        assert_eq!(written_sic(main), 2.0);
    }

    /// A black emissive glows nowhere in voxcore at any strength. Voxel Max
    /// would glow its slot in the base color, so the slot reads strength 0,
    /// which looks the same.
    #[test]
    fn a_black_emissive_writes_a_slot_glowing_nowhere() {
        let main = emissive_main(color_floats("#808080FF"), [0.0; 3], 1.0);
        assert_eq!(written_sic(main), 0.0);
    }

    /// A black emissive over a black base color glows in its base color, so its
    /// slot keeps its strength, as a Voxel Max document's black cell on a
    /// glowing slot does.
    #[test]
    fn a_black_emissive_over_a_black_base_keeps_its_strength() {
        let main = emissive_main(color_floats("#000000FF"), [0.0; 3], 2.0);
        assert_eq!(written_sic(main), 2.0);
    }

    /// A glowing emissive that differs from the base color cannot be written:
    /// Voxel Max would glow in the base color, changing how the model looks.
    #[test]
    fn a_glowing_emissive_that_differs_from_the_base_color_errors() {
        let main = emissive_main(color_floats("#808080FF"), [1.0; 3], 2.0);
        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &options(VMaxColorFormat::All),
        )
        .unwrap_err();
        assert!(error.to_string().contains("glows"), "{error}");
    }

    /// A state whose one palette binds `baseColor` over `colors` and each of
    /// `properties` over its value pool, with one material per row of
    /// `materials` drawing those value ids in property order, and one object
    /// placing a voxel of each material along x.
    fn material_main(
        colors: &[&str],
        properties: Vec<(&str, VoxValuePool)>,
        materials: &[&[u32]],
    ) -> VoxMain<()> {
        let mut main = VoxMain::default();
        let colors_id = main.retain_value_pool(
            VoxValuePool::vec_4_float(colors.iter().map(|hex| color_floats(hex)).collect())
                .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors_id)
            .unwrap();
        for (name, value_pool) in properties {
            let value_pool_id = main.retain_value_pool(value_pool);
            palette
                .retain_property(name.to_owned(), value_pool_id)
                .unwrap();
        }
        for value_ids in materials {
            palette
                .retain_material(value_ids.iter().map(|&id| U32Id::from_u32(id)).collect())
                .unwrap();
        }
        let palette_id = main.retain_palette(palette).unwrap();
        let count = u32::try_from(materials.len()).unwrap();
        let voxels: Vec<_> = (0..count).map(|x| ([x, 0, 0], x)).collect();
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(count, 1, 1),
            &voxels,
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();
        main
    }

    /// The material slot each 1-based color cell's voxels write.
    fn slots_by_cell(file: &VMaxFile) -> BTreeMap<u8, u8> {
        contents_voxels(file, "contents.vmaxb")
            .iter()
            .map(|voxel| (voxel.color_idx, voxel.material_idx))
            .collect()
    }

    /// The metallic and roughness coefficients of the first `count` slots the
    /// written palette lists.
    fn slot_coefficients(file: &VMaxFile, count: usize) -> Vec<(f64, f64)> {
        file.palette_settings_files["palette1.settings.vmaxpsb"]
            .materials
            .iter()
            .take(count)
            .map(|material| (material.mc, material.rc))
            .collect()
    }

    fn assert_coefficients(actual: &[(f64, f64)], expected: &[(f64, f64)]) {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert_eq!(actual.len(), expected.len());
        for (&(mc, rc), &(want_mc, want_rc)) in actual.iter().zip(expected) {
            assert!(close(mc, want_mc) && close(rc, want_rc), "{actual:?}");
        }
    }

    /// A palette pooling each material property's distinct values apart, as
    /// `sdf-doc voxelize` writes one, takes a slot per distinct set of values,
    /// and every material keeps the values it draws.
    #[test]
    fn lays_out_per_property_value_pools_in_slots() {
        let main = material_main(
            &["#D9B04EFF", "#5C4033FF", "#2A4FB0FF"],
            vec![
                (METALLIC, VoxValuePool::float(vec![1.0, 0.0]).unwrap()),
                (
                    ROUGHNESS,
                    VoxValuePool::float(vec![0.25, 0.6, 0.05]).unwrap(),
                ),
            ],
            &[&[0, 0, 0], &[1, 1, 1], &[2, 1, 2]],
        );

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        assert_eq!(
            slots_by_cell(&file),
            BTreeMap::from([(1, 0), (2, 1), (3, 2)])
        );
        assert_coefficients(
            &slot_coefficients(&file, 3),
            &[(0.9, 0.3), (0.1, 0.58), (0.1, 0.14)],
        );
    }

    /// Materials drawing equal material values through different value ids
    /// share one slot.
    #[test]
    fn merges_equal_material_values_into_one_slot() {
        let main = material_main(
            &["#FF0000FF", "#00FF00FF"],
            vec![
                (METALLIC, VoxValuePool::float(vec![0.5]).unwrap()),
                (ROUGHNESS, VoxValuePool::float(vec![0.3, 0.3]).unwrap()),
            ],
            &[&[0, 0, 0], &[1, 0, 1]],
        );

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        assert_eq!(slots_by_cell(&file), BTreeMap::from([(1, 0), (2, 0)]));
        assert_coefficients(&slot_coefficients(&file, 1), &[(0.5, 0.34)]);
    }

    /// The value pool and value ids each property of the first palette binds,
    /// in property order.
    fn palette_bindings<T: VoxExt>(
        main: &VoxMain<T>,
    ) -> Vec<(String, U32Id<BVoxValuePool>, Vec<U32Id<BVoxValuePoolValue>>)> {
        let palette = main.palette(U32Id::from_u32(0)).unwrap();
        palette
            .iter_properties()
            .map(|(property_id, property)| {
                let value_ids = palette
                    .iter_materials()
                    .map(|material_id| palette.value_id(material_id, property_id).unwrap())
                    .collect();
                (property.name.clone(), property.value_pool_id, value_ids)
            })
            .collect()
    }

    /// A palette already holding one value per slot keeps its value pools and
    /// value ids, so it writes as it did before, even with two slots holding
    /// equal values.
    #[test]
    fn leaves_a_palette_in_slots_as_it_is() {
        let main = material_main(
            &["#FF0000FF", "#00FF00FF", "#0000FFFF"],
            vec![
                (METALLIC, VoxValuePool::float(vec![0.0, 1.0]).unwrap()),
                (ROUGHNESS, VoxValuePool::float(vec![0.5, 0.5]).unwrap()),
            ],
            &[&[0, 1, 1], &[1, 0, 0], &[2, 1, 1]],
        );
        let before = palette_bindings(&main);

        let main = to_vmax_vox_main(main).unwrap();

        assert_eq!(palette_bindings(&main), before);
    }

    /// A material glowing nowhere leaves the glowing slot it shares for a slot
    /// at strength 0, and the material glowing in its base color keeps its
    /// strength.
    #[test]
    fn moves_a_material_glowing_nowhere_off_a_glowing_slot() {
        let main = material_main(
            &["#808080FF", "#FF0000FF"],
            vec![
                (
                    EMISSIVE_COLOR,
                    VoxValuePool::vec_3_float(vec![[0.0; 3], [1.0, 0.0, 0.0]]).unwrap(),
                ),
                (EMISSIVE_STRENGTH, VoxValuePool::float(vec![1.0]).unwrap()),
            ],
            &[&[0, 0, 0], &[1, 1, 0]],
        );

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        assert_eq!(slots_by_cell(&file), BTreeMap::from([(1, 0), (2, 1)]));
        let materials = &file.palette_settings_files["palette1.settings.vmaxpsb"].materials;
        assert_eq!((materials[0].sic, materials[1].sic), (0.0, 1.0));
    }

    /// A state whose one palette holds ten materials, each with its own color
    /// and roughness, so the ten need ten slots, and one object per row of
    /// `object_materials` sampling those materials, each placed by its own
    /// root.
    fn crowded_main(object_materials: &[&[u32]]) -> VoxMain<()> {
        let mut main = VoxMain::default();
        let colors_id = main.retain_value_pool(
            VoxValuePool::vec_4_float(
                (0..10)
                    .map(|index| color_floats(&format!("#{:02X}0000FF", index * 20 + 10)))
                    .collect(),
            )
            .unwrap(),
        );
        let roughness_id = main.retain_value_pool(
            VoxValuePool::float((0..10).map(|index| f64::from(index) / 10.0).collect()).unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors_id)
            .unwrap();
        palette
            .retain_property(ROUGHNESS.to_owned(), roughness_id)
            .unwrap();
        for value_id in IdRange::from_len(10) {
            palette.retain_material(vec![value_id, value_id]).unwrap();
        }
        let palette_id = main.retain_palette(palette).unwrap();

        let mut root_ids = Vec::new();
        for (index, material_indices) in (0..).zip(object_materials) {
            let count = u32::try_from(material_indices.len()).unwrap();
            let voxels: Vec<_> = (0..count)
                .zip(material_indices.iter())
                .map(|(x, &material)| ([x, 0, 0], material))
                .collect();
            main.retain_object(color_object(
                palette_id,
                TyVector3U32::new(count.max(1), 1, 1),
                &voxels,
            ))
            .unwrap();
            root_ids.push(
                main.retain_hierarchy_node(object_node(
                    "o",
                    index,
                    at(f64::from(index) * 20.0, 0.0, 0.0),
                ))
                .unwrap(),
            );
        }
        main.set_root_hierarchy_node_ids(root_ids).unwrap();
        main.validate().unwrap();
        main
    }

    /// A shared palette needing more slots than Voxel Max holds splits into a
    /// palette per set of materials an object samples. Objects sampling the
    /// same set share one, each keeps its materials' values, and every voxel
    /// keeps its color and place.
    #[test]
    fn splits_a_crowded_shared_palette_per_sampled_set() {
        let main = crowded_main(&[&[0, 1, 2, 3, 4], &[5, 6, 7, 8, 9], &[4, 3, 2, 1, 0]]);
        let placed = world_voxels(&main);

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        let palettes: Vec<_> = file
            .scene_json_file
            .objects
            .iter()
            .map(|object| object.palette.as_str())
            .collect();
        assert_eq!(palettes, ["palette1.png", "palette2.png", "palette1.png"]);
        let roughness = |name: &str| -> Vec<f64> {
            file.palette_settings_files[name]
                .materials
                .iter()
                .take(5)
                .map(|material| material.rc)
                .collect()
        };
        let coefficients = |factors: [f64; 5]| factors.map(|factor| 0.1 + factor * 0.8);
        for (name, factors) in [
            ("palette1.settings.vmaxpsb", [0.0, 0.1, 0.2, 0.3, 0.4]),
            ("palette2.settings.vmaxpsb", [0.5, 0.6, 0.7, 0.8, 0.9]),
        ] {
            let written = roughness(name);
            let close = written
                .iter()
                .zip(coefficients(factors))
                .all(|(a, b)| (a - b).abs() < 1e-6);
            assert!(close, "{name}: {written:?}");
        }
        assert_eq!(world_voxels(&from_vmax_file(&file).unwrap()), placed);
    }

    /// An object without a live voxel on a crowded shared palette takes an
    /// empty split of its own rather than stopping the split.
    #[test]
    fn splits_a_crowded_palette_beside_an_empty_object() {
        let main = crowded_main(&[&[0, 1, 2, 3, 4], &[5, 6, 7, 8, 9], &[]]);
        let placed = world_voxels(&main);

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        assert_eq!(world_voxels(&from_vmax_file(&file).unwrap()), placed);
    }

    /// Splitting a palette holding a material no voxel samples would drop the
    /// material, so the palette stays whole and the write errors.
    #[test]
    fn a_crowded_palette_with_an_unsampled_material_errors() {
        let main = crowded_main(&[&[0, 1, 2, 3, 4], &[5, 6, 7, 8]]);

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap_err();

        assert!(error.to_string().contains("material slots"), "{error}");
    }

    /// A shared palette within Voxel Max's slots stays shared.
    #[test]
    fn keeps_a_shared_palette_within_the_slots() {
        let mut main = crowded_main(&[&[0, 1, 2], &[3, 4, 5]]);
        let unsampled: HashSet<_> = main
            .palette(U32Id::from_u32(0))
            .unwrap()
            .iter_materials()
            .skip(6)
            .collect();
        main.release_materials(U32Id::from_u32(0), &unsampled)
            .unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        assert!(
            file.scene_json_file
                .objects
                .iter()
                .all(|object| object.palette == "palette1.png")
        );
    }

    /// Voxel Max places an object by `T(t_p) * R * S` over its workspace grid,
    /// so the voxel at grid position `v` renders centered at
    /// `t_p + R*S*(v + 0.5)`, turning about the grid's origin rather than the
    /// content center. Each voxel of a rotated object renders where its node
    /// places it.
    #[test]
    fn writes_a_rotated_object_where_voxel_max_renders_it() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(3, 2, 1),
            &[([0, 0, 0], 0), ([2, 1, 0], 0)],
        ))
        .unwrap();
        let transform = TyTransformF64::new(
            TyVector3F64::new(5.0, 7.0, -3.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::X, (-70f64).to_radians()),
            TyVector3F64::new(1.0, 1.0, 1.0),
        );
        let node_id = main
            .retain_hierarchy_node(object_node("o", 0, transform))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id]).unwrap();
        main.validate().unwrap();
        let mut placed: Vec<_> = [[0.5, 0.5, 0.5], [2.5, 1.5, 0.5]]
            .into_iter()
            .map(|center| {
                let center =
                    transform.position + transform.rotation * TyVector3F64::from_array(center);
                center.yup_to_zup()
            })
            .collect();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();

        let object = &file.scene_json_file.objects[0];
        let rotation = decode_axis_angle(object.rotation);
        let mut rendered: Vec<_> = contents_voxels(&file, &object.data)
            .iter()
            .map(|voxel| {
                let grid = TyVector3F64::from_array(voxel.position.map(f64::from))
                    + TyVector3F64::splat(0.5);
                TyVector3F64::from_array(object.position) + rotation * grid
            })
            .collect();
        let order =
            |a: &TyVector3F64, b: &TyVector3F64| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y));
        placed.sort_by(order);
        rendered.sort_by(order);
        assert_eq!(rendered.len(), placed.len());
        for (rendered, placed) in rendered.iter().zip(&placed) {
            assert!(
                (*rendered - *placed).length() < 1e-9,
                "{rendered:?} != {placed:?}"
            );
        }
    }

    /// A 6-hex source color widens to opaque RGBA: the missing alpha defaults to
    /// fully opaque rather than transparent.
    #[test]
    fn widens_rgb_source_to_opaque() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#3366CC"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        assert_eq!(
            file.palette_png_files["palette1.png"].0[0],
            [0x33, 0x66, 0xCC, 0xFF]
        );
        let reloaded = from_vmax_file(&file).unwrap();
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([0, 0, 0], [0x33, 0x66, 0xCC, 0xFF])])
        );
    }

    /// Writing a colored single-object scene, reloading it bare, then writing
    /// it again reaches a fixed point: the second synthesis reproduces the
    /// first document exactly.
    #[test]
    fn is_idempotent_for_a_colored_object() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(3.0, 4.0, 5.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file1 = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let reloaded = from_vmax_file(&file1).unwrap();
        let file2 = to_vmax_file(
            &to_vmax_vox_main(reloaded.take_ext().main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        assert_eq!(file2, file1);
    }

    /// A deep hierarchy of nested groups and object nodes round-trips: every
    /// leaf's world voxel survives the reconstructed parent chain.
    #[test]
    fn synthesizes_a_deep_hierarchy() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 1)],
        ))
        .unwrap();
        // root group 0 -> group 1 -> group 2 -> object node 3, plus object node
        // 4 hanging off group 1, so leaves sit at different depths.
        main.retain_hierarchy_nodes(vec![
            group_node("r", &[1], at(1.0, 0.0, 0.0)),
            group_node("g1", &[2, 4], at(0.0, 2.0, 0.0)),
            group_node("g2", &[3], at(0.0, 0.0, 3.0)),
            object_node("leaf", 0, at(10.0, 0.0, 0.0)),
            object_node("mid", 1, at(0.0, 20.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let reloaded = from_vmax_file(&file).unwrap();
        let red = [0xFF, 0, 0, 0xFF];
        let green = [0, 0xFF, 0, 0xFF];
        assert_eq!(
            world_voxels(&reloaded),
            BTreeSet::from([([11, 2, 3], red), ([1, 22, 0], green)])
        );
    }

    /// A group's content box is derived from its subtree, not stored: the
    /// union, in the group's own frame, of each child object's content box
    /// mapped through the placing node's transform. Two equally-sized children
    /// at different offsets union to a box spanning both.
    #[test]
    fn derives_a_group_content_box_from_its_subtree() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        // Two [2, 2, 2] objects, each tight (voxels reach both ends of every
        // axis).
        for _ in 0..2 {
            main.retain_object(color_object(
                palette_id,
                TyVector3U32::new(2, 2, 2),
                &[([0, 0, 0], 0), ([1, 1, 1], 0)],
            ))
            .unwrap();
        }
        // A root group parenting two object nodes, the second offset +10x.
        main.retain_hierarchy_nodes(vec![
            group_node("g", &[1, 2], at(0.0, 0.0, 0.0)),
            object_node("a", 0, at(0.0, 0.0, 0.0)),
            object_node("b", 1, at(10.0, 0.0, 0.0)),
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        let group = file
            .scene_json_file
            .groups
            .iter()
            .find(|g| g.name == "g")
            .expect("the root group");
        // Each object box is [0, 0, 0]..[2, 2, 2] in its node-local frame,
        // which lands on [0, -2, 0]..[2, 0, 2] on Voxel Max's Z-up axes,
        // centered on [1, -1, 1]. Object b is shifted +10x, so the union spans
        // x [0, 12], y [-2, 0], and z [0, 2], centered on [6, -1, 1] with
        // half-extents [6, 1, 1].
        assert_eq!(group.center, [6.0, -1.0, 1.0]);
        assert_eq!(group.bounds_min, Some([-6.0, -1.0, -1.0]));
        assert_eq!(group.bounds_max, Some([6.0, 1.0, 1.0]));
    }

    /// An empty object synthesizes to a scene object with empty contents and a
    /// content box framed on its grid. Reloading keeps the `[3, 4, 5]` build
    /// volume as the object's grid; with no live voxels its derived runtime
    /// extent is empty.
    #[test]
    fn synthesizes_an_empty_object_with_bounds() {
        let mut main = VoxMain::default();
        // An empty object holds no voxels; its grid is the [3, 4, 5] build
        // volume.
        let object = VoxObject::new(String::new(), TyVector3U32::new(3, 4, 5)).unwrap();
        main.retain_object(object).unwrap();
        main.retain_hierarchy_node(object_node("empty", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        assert_eq!(file.scene_json_file.objects.len(), 1);
        assert!(contents_voxels(&file, "contents.vmaxb").is_empty());
        // The content box frames the [3, 4, 5] build volume, [3, 5, 4] on
        // Voxel Max's Z-up axes, centered in the 256 workspace: e_c at its
        // center, e_ma the half-extents.
        assert_eq!(file.scene_json_file.objects[0].center, [127.5, 127.5, 2.0]);
        assert_eq!(
            file.scene_json_file.objects[0].bounds_max,
            Some([1.5, 2.5, 2.0])
        );
        // Reloading keeps the [3, 4, 5] build volume as the object's grid; with
        // no live voxels its derived runtime extent is empty.
        let reloaded = from_vmax_file(&file).unwrap();
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        assert_eq!(
            reloaded.object(object_id).unwrap().bounds(),
            TyVector3U32::new(3, 4, 5)
        );
        assert_eq!(reloaded.object(object_id).unwrap().live_extent(), None);
    }

    /// A fully transparent color round-trips as a real color at image index 0
    /// rather than being mistaken for the trailing terminator.
    #[test]
    fn round_trips_a_transparent_color() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#11223300"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap();
        // The color sits at index 0 (cell 0); the terminator is at the end.
        assert_eq!(
            file.palette_png_files["palette1.png"].0[0],
            [0x11, 0x22, 0x33, 0]
        );
        assert_eq!(file.palette_png_files["palette1.png"].0[255], [0, 0, 0, 0]);
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

    /// The color table is the `baseColor` value pool whole, so a pool past the
    /// budget errors even when the voxels use only its low cells, such as a
    /// MagicaVoxel source's fixed 256-entry palette. Nothing is truncated.
    #[test]
    fn a_palette_past_the_color_budget_errors() {
        let mut main = VoxMain::default();
        let hexes: Vec<String> = (0..256).map(|i| format!("#{:06X}FF", i)).collect();
        let refs: Vec<&str> = hexes.iter().map(String::as_str).collect();
        let palette_id = retain_rgba_palette(&mut main, &refs);
        // One voxel references a low cell; the other 255 cells are padding.
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 5)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let error = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &VMaxWriteOptions::default(),
        )
        .unwrap_err();
        assert!(error.to_string().contains("256 colors"), "{error}");
    }

    /// A color-only object keeps its colors in plist mode, where the colors
    /// ride in the settings sidecar rather than an image, instead of reloading
    /// as white.
    #[test]
    fn synthesizes_a_color_only_object_in_plist() {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF", "#00FF00FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(2, 1, 1),
            &[([0, 0, 0], 0), ([1, 0, 0], 1)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();

        let file = to_vmax_file(
            &to_vmax_vox_main(main).unwrap(),
            &options(VMaxColorFormat::Plist),
        )
        .unwrap();
        // No image in plist mode; the colors must still survive via the
        // sidecar.
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

    /// A minimal bare state: one red voxel placed by one object node.
    fn one_object_state() -> VoxMain<()> {
        let mut main = VoxMain::default();
        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);
        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();
        main.retain_hierarchy_node(object_node("o", 0, at(0.0, 0.0, 0.0)))
            .unwrap();
        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();
        main.validate().unwrap();
        main
    }

    /// `SceneCameraSource::Camera` writes the given scene camera, replacing
    /// whatever the path would otherwise produce.
    #[test]
    fn scene_camera_camera_overrides_the_scene_camera() {
        let main = one_object_state();
        let camera = VMaxSceneCamera {
            z: 123.0,
            ..Default::default()
        };
        let options = VMaxWriteOptions {
            scene_camera: SceneCameraSource::Camera(camera),
            ..Default::default()
        };
        let file = to_vmax_file(&to_vmax_vox_main(main).unwrap(), &options).unwrap();
        assert_eq!(file.scene_json_file.cam, Some(camera));
    }

    /// A synthesized ext carries the neutral camera, so `Ext` writes the same
    /// camera `Empty` does.
    #[test]
    fn scene_camera_ext_writes_the_synthesized_camera() {
        let main = to_vmax_vox_main(source_state()).unwrap();

        assert_eq!(main.ext().scene.cam, Some(SYNTH_CAMERA));

        let options = VMaxWriteOptions {
            scene_camera: SceneCameraSource::Ext,
            ..Default::default()
        };

        let file = to_vmax_file(&main, &options).unwrap();

        assert_eq!(file.scene_json_file.cam, Some(SYNTH_CAMERA));
    }

    #[test]
    fn synthesizes_an_entry_per_node_linked_to_its_parent() {
        let mut main = VoxMain::default();

        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);

        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();

        main.retain_hierarchy_nodes(vec![
            group_node("g", &[1, 2], at(0.0, 0.0, 0.0)),
            object_node("a", 0, at(0.0, 0.0, 0.0)),
            object_node("b", 0, at(5.0, 0.0, 0.0)),
        ])
        .unwrap();

        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();

        let main = to_vmax_vox_main(main).unwrap();

        let ext = main.ext();

        let node = |index: u32| &ext.hierarchy_nodes[&U32Id::from_u32(index)];

        assert_eq!(ext.hierarchy_nodes.len(), 3);

        assert_ne!(node(1).id, node(2).id);

        assert_ne!(node(1).index, node(2).index);

        assert_eq!(node(0).alignment, "f");

        assert_eq!(
            ext.palettes,
            BTreeMap::from([(U32Id::from_u32(0), VMaxExtPalette::default())])
        );

        assert_eq!(ext.object_states.len(), 1);

        assert_eq!(ext.scene.v, FALLBACK_CONTENT_VERSION);

        assert_eq!(ext.scene.cam, Some(SYNTH_CAMERA));

        // The parent rides in the hierarchy, so the written objects link to
        // the group through it.
        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();

        assert_eq!(file.scene_json_file.groups[0].parent_id, None);

        for object in &file.scene_json_file.objects {
            assert_eq!(object.parent_id.as_deref(), Some(node(0).id.as_str()));
        }
    }

    #[test]
    fn clones_a_subtree_shared_by_two_parents() {
        let mut main = VoxMain::default();

        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);

        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();

        // Node 2 is a child of both group nodes.
        main.retain_hierarchy_nodes(vec![
            group_node("a", &[2], at(0.0, 0.0, 0.0)),
            group_node("b", &[2], at(100.0, 0.0, 0.0)),
            object_node("c", 0, at(1.0, 0.0, 0.0)),
        ])
        .unwrap();

        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();

        let main = to_vmax_vox_main(main).unwrap();

        assert_eq!(main.hierarchy_node_count(), 4);

        let b = main.hierarchy_node(U32Id::from_u32(1)).unwrap();

        assert_eq!(b.child_node_ids, vec![U32Id::from_u32(3)]);

        let clone = main.hierarchy_node(U32Id::from_u32(3)).unwrap();

        assert_eq!(clone.name, "c");

        assert_eq!(clone.transform, at(1.0, 0.0, 0.0));

        assert_eq!(
            clone.child_object_ids,
            vec![U32Id::<BVoxObject>::from_u32(0)]
        );

        assert_eq!(main.object_count(), 1);

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();

        let clone_id = main.ext().hierarchy_nodes[&U32Id::from_u32(3)].id.as_str();

        let written = file
            .scene_json_file
            .objects
            .iter()
            .find(|object| object.id == clone_id)
            .expect("the clone writes as an object");

        assert_eq!(
            written.parent_id.as_deref(),
            Some(main.ext().hierarchy_nodes[&U32Id::from_u32(1)].id.as_str())
        );
    }

    #[test]
    fn clones_a_node_that_is_root_and_child() {
        let mut main = VoxMain::default();

        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);

        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();

        main.retain_hierarchy_nodes(vec![
            group_node("a", &[1], at(100.0, 0.0, 0.0)),
            object_node("c", 0, at(1.0, 0.0, 0.0)),
        ])
        .unwrap();

        main.set_root_hierarchy_node_ids(vec![
            U32Id::<BVoxHierarchyNode>::from_u32(0),
            U32Id::<BVoxHierarchyNode>::from_u32(1),
        ])
        .unwrap();

        let main = to_vmax_vox_main(main).unwrap();

        assert_eq!(main.hierarchy_node_count(), 3);

        assert_eq!(
            main.root_hierarchy_node_ids(),
            [U32Id::from_u32(0), U32Id::from_u32(2)]
        );

        let file = to_vmax_file(&main, &VMaxWriteOptions::default()).unwrap();

        let clone_id = main.ext().hierarchy_nodes[&U32Id::from_u32(2)].id.as_str();

        let written = file
            .scene_json_file
            .objects
            .iter()
            .find(|object| object.id == clone_id)
            .expect("the clone writes as an object");

        assert_eq!(written.parent_id, None);
    }

    #[test]
    fn releases_a_node_no_root_reaches() {
        let mut main = VoxMain::default();

        let palette_id = retain_rgba_palette(&mut main, &["#FF0000FF"]);

        main.retain_object(color_object(
            palette_id,
            TyVector3U32::new(1, 1, 1),
            &[([0, 0, 0], 0)],
        ))
        .unwrap();

        // Node 1 parents node 2, and neither is a root or a root's child.
        main.retain_hierarchy_nodes(vec![
            object_node("rooted", 0, at(0.0, 0.0, 0.0)),
            group_node("orphan", &[2], at(50.0, 0.0, 0.0)),
            object_node("orphaned", 0, at(0.0, 0.0, 0.0)),
        ])
        .unwrap();

        main.set_root_hierarchy_node_ids(vec![U32Id::<BVoxHierarchyNode>::from_u32(0)])
            .unwrap();

        let main = to_vmax_vox_main(main).unwrap();

        assert_eq!(main.hierarchy_node_count(), 1);

        assert_eq!(main.ext().hierarchy_nodes.len(), 1);
    }
}
