use crate::Result;
use branded_id::{U32Id, UsizeId};
use std::collections::HashMap;
use voxcore::{
    BVoxLayer, BVoxMaterial, BVoxValuePoolValue, BVoxVoxel, VoxEffectivePalette, VoxMain,
    VoxObject, VoxValuePool,
};

/// The distinct flattened materials an object's voxels sample, the set the
/// material atlas bakes.
///
/// The object's layers merge per property name by the format's override rule,
/// resolved once as the effective palette: each property reads through the
/// last layer whose palette supplies its name. Two voxels share a texel
/// exactly when they sample the same material in every winning layer, so a
/// voxel's identity key is the tuple of its materials over those layers,
/// deduplicated first seen in raster order. The bake gives each distinct key
/// one atlas texel, [`len`](Self::len) of them, and each voxel's mesh faces
/// sample the texel of the key it uses through
/// [`material_index`](Self::material_index). With no winning layers every
/// live voxel shares the one empty-key texel, which bakes spec defaults.
pub(crate) struct UsedMaterials<'a> {
    /// The object's per-name winning suppliers.
    effective: VoxEffectivePalette<'a>,

    /// The distinct layers that won at least one property, first seen over
    /// the effective palette's stable listing. Each key tuple lists its
    /// materials in this order.
    identity_layer_ids: Vec<U32Id<BVoxLayer>>,

    /// Distinct identity keys in first-seen raster order, one per atlas
    /// texel, each the tuple of materials a voxel samples over
    /// [`identity_layer_ids`](Self::identity_layer_ids).
    keys: Vec<Vec<U32Id<BVoxMaterial>>>,

    /// Per live voxel, the position in [`keys`](Self::keys), and so the atlas
    /// texel, of the identity key it samples.
    index_of: HashMap<U32Id<BVoxVoxel>, u32>,
}

impl<'a> UsedMaterials<'a> {
    /// Number of distinct flattened materials.
    pub(crate) fn len(&self) -> usize {
        self.keys.len()
    }

    /// The atlas texel `voxel` samples, its identity key's position in the
    /// used set, or `None` if `voxel` is not a live voxel of the resolved
    /// object.
    pub(crate) fn material_index(&self, voxel: U32Id<BVoxVoxel>) -> Option<u32> {
        self.index_of.get(&voxel).copied()
    }

    /// What the texel at `index` draws for the property named `key`: the
    /// value pool the winning supplier draws from and the value id into it,
    /// read from the texel's material in the winning layer. `None` when
    /// `index` is out of range or no layer supplies `key`.
    pub(crate) fn attribute(
        &self,
        index: usize,
        key: &str,
    ) -> Option<(&'a VoxValuePool, U32Id<BVoxValuePoolValue>)> {
        let key_material_ids = self.keys.get(index)?;
        let property_id = self.effective.property_id_by_name(key)?;
        let property = self
            .effective
            .property(property_id)
            .expect("a resolved name identifies one of the effective palette's properties");
        let position = self
            .identity_layer_ids
            .iter()
            .position(|&layer_id| layer_id == property.layer_id())
            .expect("a winning layer is one of the identity layers");
        let value_id = property.value_id(key_material_ids[position])?;
        Some((property.value_pool(), value_id))
    }
}

/// Resolves the distinct flattened materials `object` uses, its layers merged
/// per property name by the format's override rule, first seen in raster
/// order. Errors when a layer references a palette `state` does not hold.
pub(crate) fn resolve_used_materials<'a>(
    state: &'a VoxMain,
    object: &'a VoxObject,
) -> Result<UsedMaterials<'a>> {
    let effective = state.effective_palette(object)?;

    let mut identity_layer_ids: Vec<U32Id<BVoxLayer>> = Vec::new();
    for index in 0..effective.property_count() {
        let property = effective
            .property(UsizeId::from_usize(index))
            .expect("property ids below the count resolve");

        if !identity_layer_ids.contains(&property.layer_id()) {
            identity_layer_ids.push(property.layer_id());
        }
    }

    let mut keys: Vec<Vec<U32Id<BVoxMaterial>>> = Vec::new();
    let mut lookup: HashMap<Vec<U32Id<BVoxMaterial>>, u32> = HashMap::new();
    let mut index_of: HashMap<U32Id<BVoxVoxel>, u32> = HashMap::new();

    for voxel_id in object.iter_live() {
        let key: Vec<U32Id<BVoxMaterial>> = identity_layer_ids
            .iter()
            .map(|&layer_id| {
                object
                    .voxel_material(voxel_id, layer_id)
                    .expect("a live voxel samples a material in each of the object's layers")
            })
            .collect();

        let index = match lookup.get(&key) {
            Some(&index) => index,
            None => {
                let index = keys.len() as u32;
                keys.push(key.clone());
                lookup.insert(key, index);
                index
            }
        };

        index_of.insert(voxel_id, index);
    }

    Ok(UsedMaterials {
        effective,
        identity_layer_ids,
        keys,
        index_of,
    })
}

#[cfg(test)]
mod tests {
    use crate::resolve_used_materials;
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{
        BVoxPalette, BVoxValuePool, BVoxValuePoolValue, VoxBound, VoxMain, VoxObject, VoxPalette,
        VoxValuePool, VoxValuePoolValueRef,
    };

    /// The branded value id `index`.
    fn value_id(index: u32) -> U32Id<BVoxValuePoolValue> {
        U32Id::from_u32(index)
    }

    /// Adds an unbounded `float` value pool holding `values` and returns its
    /// id.
    fn float_pool(state: &mut VoxMain, values: Vec<f64>) -> U32Id<BVoxValuePool> {
        state.add_value_pool(VoxValuePool::float(VoxBound::None, VoxBound::None, values).unwrap())
    }

    /// Adds a palette binding each `(name, value id)` entry on `pool_id` and
    /// one material drawing the listed value ids, and returns its id.
    fn palette_over(
        state: &mut VoxMain,
        pool_id: U32Id<BVoxValuePool>,
        entries: &[(&str, u32)],
    ) -> U32Id<BVoxPalette> {
        let mut palette = VoxPalette::default();
        for &(name, _) in entries {
            palette
                .add_property(name.to_owned(), pool_id, value_id(0))
                .unwrap();
        }
        palette
            .add_material(entries.iter().map(|&(_, index)| value_id(index)).collect())
            .unwrap();
        state.add_palette(palette).unwrap()
    }

    /// A one-voxel object layering the given palettes in order, its voxel
    /// live and sampling material 0 in every layer.
    fn object_over(palette_ids: &[U32Id<BVoxPalette>]) -> VoxObject {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        for &palette_id in palette_ids {
            object.add_layer(palette_id, U32Id::from_u32(0));
        }
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        let sample_ids = vec![U32Id::from_u32(0); palette_ids.len()];
        object.retain_voxel(voxel_id, &sample_ids).unwrap();
        object
    }

    #[test]
    fn each_property_reads_through_its_winning_layer() {
        // The worked flatten example: layers binding {a, b}, {c}, {b, c}. The
        // winners are a from the first layer, b and c from the third.
        let mut state = VoxMain::default();
        let pool_id = float_pool(&mut state, vec![0.1, 0.2, 0.3, 0.4, 0.5]);
        let first_id = palette_over(&mut state, pool_id, &[("a", 0), ("b", 1)]);
        let second_id = palette_over(&mut state, pool_id, &[("c", 2)]);
        let third_id = palette_over(&mut state, pool_id, &[("b", 3), ("c", 4)]);
        let object = object_over(&[first_id, second_id, third_id]);

        let used = resolve_used_materials(&state, &object).unwrap();
        assert_eq!(used.len(), 1);

        let read = |key: &str| {
            let (pool, value_id) = used.attribute(0, key).unwrap();
            pool.value(value_id).unwrap()
        };
        assert_eq!(read("a"), VoxValuePoolValueRef::Float(0.1));
        assert_eq!(read("b"), VoxValuePoolValueRef::Float(0.4));
        assert_eq!(read("c"), VoxValuePoolValueRef::Float(0.5));
        assert_eq!(used.attribute(0, "d"), None);
    }

    #[test]
    fn properties_pull_from_their_own_palettes_and_pools() {
        // Two layers on different palettes over different pools: each name
        // resolves to its own palette's pool and value.
        let mut state = VoxMain::default();
        let first_pool_id = float_pool(&mut state, vec![0.25]);
        let second_pool_id = float_pool(&mut state, vec![0.75]);
        let first_id = palette_over(&mut state, first_pool_id, &[("a", 0)]);
        let second_id = palette_over(&mut state, second_pool_id, &[("b", 0)]);
        let object = object_over(&[first_id, second_id]);

        let used = resolve_used_materials(&state, &object).unwrap();
        assert_eq!(used.len(), 1);

        let (a_pool, a_value_id) = used.attribute(0, "a").unwrap();
        let (b_pool, b_value_id) = used.attribute(0, "b").unwrap();
        assert_eq!(
            a_pool.value(a_value_id),
            Some(VoxValuePoolValueRef::Float(0.25))
        );
        assert_eq!(
            b_pool.value(b_value_id),
            Some(VoxValuePoolValueRef::Float(0.75))
        );
    }

    #[test]
    fn a_property_less_layer_supplies_nothing() {
        // The later layer's palette binds no properties, so it wins nothing
        // and its samples never split texels: two voxels differing only in it
        // share one texel, and the earlier layer keeps its name.
        let mut state = VoxMain::default();
        let pool_id = float_pool(&mut state, vec![0.5]);
        let supplying_id = palette_over(&mut state, pool_id, &[("a", 0)]);

        let mut plain = VoxPalette::default();
        plain.add_material(vec![]).unwrap();
        plain.add_material(vec![]).unwrap();
        let plain_id = state.add_palette(plain).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        object.add_layer(supplying_id, U32Id::from_u32(0));
        object.add_layer(plain_id, U32Id::from_u32(0));
        for (x, plain_material) in [(0, 0), (1, 1)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .retain_voxel(
                    voxel_id,
                    &[U32Id::from_u32(0), U32Id::from_u32(plain_material)],
                )
                .unwrap();
        }

        let used = resolve_used_materials(&state, &object).unwrap();
        assert_eq!(used.len(), 1);
        assert_eq!(
            used.attribute(0, "a")
                .map(|(pool, value_id)| pool.value(value_id)),
            Some(Some(VoxValuePoolValueRef::Float(0.5)))
        );
    }

    #[test]
    fn a_layerless_object_shares_one_empty_key_texel() {
        let state = VoxMain::default();
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(2, 1, 1)).unwrap();
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[]).unwrap();
        }

        let used = resolve_used_materials(&state, &object).unwrap();
        assert_eq!(used.len(), 1);
        for x in 0..2 {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            assert_eq!(used.material_index(voxel_id), Some(0));
        }
        assert_eq!(used.attribute(0, "baseColorFactor"), None);
    }

    #[test]
    fn distinct_winning_layer_tuples_split_texels() {
        // Two layers each binding a name over a two-material palette: voxels
        // share a texel exactly when they sample the same material in both.
        let mut state = VoxMain::default();
        let pool_id = float_pool(&mut state, vec![0.0, 1.0]);

        let mut palette = VoxPalette::default();
        palette
            .add_property("a".to_owned(), pool_id, value_id(0))
            .unwrap();
        palette.add_material(vec![value_id(0)]).unwrap();
        palette.add_material(vec![value_id(1)]).unwrap();
        let first_id = state.add_palette(palette).unwrap();

        let mut second = VoxPalette::default();
        second
            .add_property("b".to_owned(), pool_id, value_id(0))
            .unwrap();
        second.add_material(vec![value_id(0)]).unwrap();
        second.add_material(vec![value_id(1)]).unwrap();
        let second_id = state.add_palette(second).unwrap();

        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        object.add_layer(first_id, U32Id::from_u32(0));
        object.add_layer(second_id, U32Id::from_u32(0));
        // Samples per voxel: (0, 0), (0, 1), (0, 0), so two distinct tuples.
        for (x, second_material) in [(0, 0), (1, 1), (2, 0)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .retain_voxel(
                    voxel_id,
                    &[U32Id::from_u32(0), U32Id::from_u32(second_material)],
                )
                .unwrap();
        }

        let used = resolve_used_materials(&state, &object).unwrap();
        assert_eq!(used.len(), 2);
        let index_at = |x| {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            used.material_index(voxel_id).unwrap()
        };
        assert_eq!(index_at(0), 0);
        assert_eq!(index_at(1), 1);
        assert_eq!(index_at(2), 0);

        // The split texels differ only in `b`, the second layer's name.
        let read = |index: usize, key: &str| {
            let (pool, value_id) = used.attribute(index, key).unwrap();
            pool.value(value_id).unwrap()
        };
        assert_eq!(read(0, "a"), read(1, "a"));
        assert_eq!(read(0, "b"), VoxValuePoolValueRef::Float(0.0));
        assert_eq!(read(1, "b"), VoxValuePoolValueRef::Float(1.0));
    }

    #[test]
    fn a_layer_over_an_unknown_palette_errors() {
        let state = VoxMain::default();
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.add_layer(U32Id::from_u32(9), U32Id::from_u32(0));

        assert!(resolve_used_materials(&state, &object).is_err());
    }
}
