use crate::{Result, operations::object::link_objects};
use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxValuePool, Error as VoxError, VoxExt,
    VoxHierarchyNode, VoxMain, VoxState,
};

type ObjectId = U32Id<BVoxObject>;

type PaletteId = U32Id<BVoxPalette>;

type ValuePoolId = U32Id<BVoxValuePool>;

/// Appends a copy of each `source` object in `object_ids` and returns the
/// copies' ids. Each source palette a copy references is appended once, in
/// source order, with the value pools it draws from, even when an equal one
/// exists. `parent_id` places every copy when given. Otherwise each copy gets
/// a new root node named for it with an identity transform. Errors, changing
/// nothing, when an id is not one of `source`'s objects or `parent_id` is not
/// one of the main's nodes.
pub fn add_objects<T: VoxExt>(
    main: &mut VoxMain<T>,
    source: &VoxState,
    object_ids: &[ObjectId],
    parent_id: Option<U32Id<BVoxHierarchyNode>>,
) -> Result<Vec<ObjectId>> {
    let mut palette_ids: HashSet<PaletteId> = HashSet::new();

    for &object_id in object_ids {
        let Some(object) = source.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        palette_ids.extend(object.iter_layers().map(|(_, palette_id)| palette_id));
    }

    if let Some(parent_id) = parent_id
        && main.hierarchy_node(parent_id).is_none()
    {
        return Err(VoxError::UnknownHierarchyNode { node_id: parent_id }.into());
    }

    let value_pool_ids: HashSet<ValuePoolId> = source
        .iter_palettes()
        .filter(|(palette_id, _)| palette_ids.contains(palette_id))
        .flat_map(|(_, palette)| {
            palette
                .iter_properties()
                .map(|(_, property)| property.value_pool_id)
        })
        .collect();

    let mut value_pool_id_map: HashMap<ValuePoolId, ValuePoolId> = HashMap::new();

    for (value_pool_id, value_pool) in source.iter_value_pools() {
        if value_pool_ids.contains(&value_pool_id) {
            let new_id = main.retain_value_pool(value_pool.clone());

            value_pool_id_map.insert(value_pool_id, new_id);
        }
    }

    let mut palette_id_map: HashMap<PaletteId, PaletteId> = HashMap::new();

    for (palette_id, palette) in source.iter_palettes() {
        if !palette_ids.contains(&palette_id) {
            continue;
        }

        let mut palette = palette.clone();

        palette.relabel_value_pools(|value_pool_id| {
            *value_pool_id_map
                .get(&value_pool_id)
                .expect("every value pool a copied palette draws from is copied")
        });

        palette_id_map.insert(palette_id, main.retain_palette(palette)?);
    }

    let mut copy_ids = Vec::with_capacity(object_ids.len());

    for &object_id in object_ids {
        let mut copy = source
            .object(object_id)
            .expect("object_ids are checked above")
            .clone();

        copy.relabel_layer_palettes(|palette_id| {
            *palette_id_map
                .get(&palette_id)
                .expect("every palette a copied object references is copied")
        });

        copy_ids.push(main.retain_object(copy)?);
    }

    if let Some(parent_id) = parent_id {
        link_objects(main, &copy_ids, parent_id)?;

        return Ok(copy_ids);
    }

    let mut root_ids = main.root_hierarchy_node_ids().to_vec();

    for &copy_id in &copy_ids {
        let name = main
            .object(copy_id)
            .expect("a copy was just retained")
            .name()
            .to_owned();

        let node = VoxHierarchyNode {
            name,
            child_object_ids: vec![copy_id],
            ..Default::default()
        };

        root_ids.push(main.retain_hierarchy_node(node)?);
    }

    main.set_root_hierarchy_node_ids(root_ids)?;

    Ok(copy_ids)
}

#[cfg(test)]
mod tests {
    use crate::{operations::object::add_objects, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{
        BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
        VoxValuePool,
    };

    /// Retains a one-value pool and a palette with one material drawing it.
    fn palette(main: &mut VoxMain, color: [f64; 4]) -> (U32Id<BVoxPalette>, U32Id<BVoxMaterial>) {
        let value_pool = VoxValuePool::vec_4_float(vec![color]).unwrap();

        let value_pool_id = main.retain_value_pool(value_pool);

        let mut palette = VoxPalette::default();
        palette
            .retain_property("baseColor".to_owned(), value_pool_id)
            .unwrap();
        let material_id = palette.retain_material(vec![U32Id::from_u32(0)]).unwrap();

        (main.retain_palette(palette).unwrap(), material_id)
    }

    /// A `1 x 1 x 1` object with one live voxel sampling `material_id` of
    /// `palette_id`.
    fn object(
        main: &mut VoxMain,
        name: &str,
        (palette_id, material_id): (U32Id<BVoxPalette>, U32Id<BVoxMaterial>),
    ) -> U32Id<BVoxObject> {
        let mut object = VoxObject::new(name.to_owned(), TyVector3U32::splat(1)).unwrap();
        object.retain_layer(palette_id).unwrap();
        let voxel_id = object.voxel_id(TyVector3U32::ZERO).unwrap();
        object.retain_voxel(voxel_id, &[material_id]).unwrap();

        main.retain_object(object).unwrap()
    }

    /// Palettes `red`, `green`, and `blue`, and objects `a` on blue, `b` on
    /// red, and `c` on green.
    fn source() -> VoxMain {
        let mut source = VoxMain::default();

        let red = palette(&mut source, [1.0, 0.0, 0.0, 1.0]);
        let green = palette(&mut source, [0.0, 1.0, 0.0, 1.0]);
        let blue = palette(&mut source, [0.0, 0.0, 1.0, 1.0]);

        object(&mut source, "a", blue);
        object(&mut source, "b", red);
        object(&mut source, "c", green);

        source
    }

    /// One palette, an object `scene` on it, and a root `house` placing it.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main = VoxMain::default();

        let white = palette(&mut main, [1.0, 1.0, 1.0, 1.0]);

        let scene_id = object(&mut main, "scene", white);

        let house = VoxHierarchyNode {
            name: "house".to_owned(),
            child_object_ids: vec![scene_id],
            ..Default::default()
        };

        let house_id = main.retain_hierarchy_node(house).unwrap();
        main.push_root_hierarchy_node_id(house_id).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn copies_the_referenced_palettes_in_source_order_under_new_roots() {
        let source = source();

        let mut main = scene();

        let copy_ids = add_objects(
            &mut main,
            source.state(),
            &[U32Id::from_u32(0), U32Id::from_u32(1)],
            None,
        )
        .unwrap();

        // Red and blue are copied in source order, after the scene's palette.
        assert_eq!(main.palette_count(), 3);
        assert_eq!(main.value_pool_count(), 3);

        let layer_palette = |object_index: usize| {
            let (_, palette_id) = main
                .object(copy_ids[object_index])
                .unwrap()
                .iter_layers()
                .next()
                .unwrap();

            palette_id
        };

        assert_eq!(layer_palette(0), U32Id::from_u32(2));
        assert_eq!(layer_palette(1), U32Id::from_u32(1));

        let root_names: Vec<&str> = main
            .root_hierarchy_node_ids()
            .iter()
            .map(|&root_id| main.hierarchy_node(root_id).unwrap().name.as_str())
            .collect();

        assert_eq!(root_names, ["house", "a", "b"]);
        assert_eq!(
            HookRecorder::events(&main),
            [
                "palette 1 retained",
                "palette 2 retained",
                "object 1 retained",
                "object 2 retained",
                "node 1 retained",
                "node 2 retained",
                "roots set",
            ]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_palette_equal_to_one_in_the_scene_is_still_copied() {
        let mut source = VoxMain::default();

        let white = palette(&mut source, [1.0, 1.0, 1.0, 1.0]);

        let a_id = object(&mut source, "a", white);

        let mut main = scene();

        add_objects(&mut main, source.state(), &[a_id], None).unwrap();

        assert_eq!(main.palette_count(), 2);
        main.validate().unwrap();
    }

    #[test]
    fn a_parent_places_every_copy() {
        let source = source();

        let mut main = scene();

        add_objects(
            &mut main,
            source.state(),
            &[U32Id::from_u32(2)],
            Some(U32Id::from_u32(0)),
        )
        .unwrap();

        assert_eq!(
            main.hierarchy_node(U32Id::from_u32(0))
                .unwrap()
                .child_object_ids,
            [U32Id::from_u32(0), U32Id::from_u32(1)]
        );
        assert_eq!(main.hierarchy_node_count(), 1);
        main.validate().unwrap();
    }

    #[test]
    fn an_unknown_object_is_an_error_that_changes_nothing() {
        let source = source();

        let mut main = scene();

        assert!(add_objects(&mut main, source.state(), &[U32Id::from_u32(9)], None).is_err());
        assert_eq!(main.palette_count(), 1);
        assert!(HookRecorder::events(&main).is_empty());
    }
}
